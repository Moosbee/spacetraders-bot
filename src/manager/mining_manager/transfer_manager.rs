use space_traders_client::models;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::error::Result;

#[derive(Debug)]
pub struct TransportTransferRequest {
    from_symbol: String,
    to_symbol: String,
    amount: i32,
    trade_symbol: models::TradeSymbol,
    extractor_contact: tokio::sync::mpsc::Sender<ExtractorTransferRequest>,
    callback: tokio::sync::oneshot::Sender<()>,
}

#[derive(Debug)]
pub struct ExtractorTransferRequest {
    from_symbol: String,
    to_symbol: String,
    amount: i32,
    trade_symbol: models::TradeSymbol,
    callback: tokio::sync::oneshot::Sender<()>,
}

#[derive(Debug)]
pub struct TransferManager {
    extraction_contacts: HashMap<String, mpsc::Sender<ExtractorTransferRequest>>,
    transportation_contacts: HashMap<String, mpsc::Sender<TransportTransferRequest>>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            extraction_contacts: HashMap::new(),
            transportation_contacts: HashMap::new(),
        }
    }

    pub async fn process_transfer(
        &mut self,
        from_extractor: &str,
        to_transporter: &str,
        symbol: models::TradeSymbol,
        amount: i32,
    ) -> Result<()> {
        let transporter = self
            .transportation_contacts
            .get(to_transporter)
            .ok_or("No transporter contact")?;

        let extractor = self
            .extraction_contacts
            .get(from_extractor)
            .ok_or("No extractor contact")?
            .clone();

        let (callback, receiver) = tokio::sync::oneshot::channel();
        let request = TransportTransferRequest {
            from_symbol: from_extractor.to_string(),
            to_symbol: to_transporter.to_string(),
            amount,
            trade_symbol: symbol,
            extractor_contact: extractor,
            callback,
        };

        if let Err(err) = transporter.send(request).await {
            self.transportation_contacts.remove(to_transporter);
            return Err(format!("Transporter no longer receives requests: {}", err)
                .as_str()
                .into());
        }

        receiver
            .await
            .map_err(|e| crate::error::Error::General(format!("Failed to get message: {}", e)))?;
        Ok(())
    }
}
