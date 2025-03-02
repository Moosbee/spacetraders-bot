use dashmap::DashMap;
use log::debug;
use space_traders_client::models;
use tokio::sync::mpsc;

use crate::types::safely_get_map;

#[derive(Debug)]
pub struct TransportTransferRequest {
    pub from_symbol: String,
    pub to_symbol: String,
    pub amount: i32,
    pub trade_symbol: models::TradeSymbol,
    pub extractor_contact: tokio::sync::mpsc::Sender<ExtractorTransferRequest>,
    pub callback: tokio::sync::oneshot::Sender<()>,
}

#[derive(Debug)]
pub struct ExtractorTransferRequest {
    pub from_symbol: String,
    pub to_symbol: String,
    pub amount: i32,
    pub trade_symbol: models::TradeSymbol,
    pub callback: tokio::sync::oneshot::Sender<Option<TransferResult>>,
}

#[derive(Debug)]
pub struct TransferResult {
    pub trade_symbol: models::TradeSymbol,
    pub units: i32,
}

#[derive(Debug)]
pub struct TransferManager {
    extraction_contacts: DashMap<String, mpsc::Sender<ExtractorTransferRequest>>,
    transportation_contacts: DashMap<String, mpsc::Sender<TransportTransferRequest>>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            extraction_contacts: DashMap::new(),
            transportation_contacts: DashMap::new(),
        }
    }

    pub fn add_extractor_contact(
        &self,
        symbol: &str,
        sender: mpsc::Sender<ExtractorTransferRequest>,
    ) {
        self.extraction_contacts.insert(symbol.to_string(), sender);
    }

    pub fn add_transportation_contact(
        &self,
        symbol: &str,
        sender: mpsc::Sender<TransportTransferRequest>,
    ) {
        debug!("Adding transportation contact for symbol: {}", symbol);
        self.transportation_contacts
            .insert(symbol.to_string(), sender);
    }

    pub fn viable(&self, from_extractor: &str, to_transporter: &str) -> bool {
        let extractor = self.valid_extractor(from_extractor);
        let transporter = self.valid_transporter(to_transporter);

        extractor && transporter
    }

    pub fn valid_extractor(&self, symbol: &str) -> bool {
        {
            self.extraction_contacts
                .remove_if(symbol, |_, c| c.is_closed());
        }
        let refi = safely_get_map(&self.extraction_contacts, &symbol.to_string());

        let is = match refi.as_ref() {
            Some(contact) => !contact.is_closed(),
            None => false,
        };

        debug!(
            "Valid extractor {} valid: {} some {} is closed {:?}",
            symbol,
            is,
            refi.is_some(),
            refi.as_ref().map(|f| f.is_closed())
        );

        is
    }

    pub fn valid_transporter(&self, symbol: &str) -> bool {
        {
            self.transportation_contacts
                .remove_if(symbol, |_, c| c.is_closed());
        }
        let refi = safely_get_map(&self.transportation_contacts, &symbol.to_string());

        let is = match refi.as_ref() {
            Some(contact) => !contact.is_closed(),
            None => false,
        };

        debug!(
            "Valid transporter {} valid: {} some {} is closed {:?}",
            symbol,
            is,
            refi.is_some(),
            refi.as_ref().map(|f| f.is_closed())
        );

        is
    }

    pub async fn process_transfer(
        &self,
        from_extractor: &str,
        to_transporter: &str,
        symbol: models::TradeSymbol,
        amount: i32,
    ) -> Result<(), Error> {
        let transporter =
            safely_get_map(&self.transportation_contacts, &to_transporter.to_string())
                .filter(|c| !c.is_closed())
                .ok_or(Error::TransporterDropped {
                    symbol: to_transporter.to_string(),
                    from: from_extractor.to_string(),
                    to: to_transporter.to_string(),
                })?;

        let extractor = safely_get_map(&self.extraction_contacts, &from_extractor.to_string())
            .filter(|c| !c.is_closed())
            .ok_or(Error::ExtractorDropped {
                symbol: from_extractor.to_string(),
                from: from_extractor.to_string(),
                to: to_transporter.to_string(),
            })?
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

        if let Err(_err) = transporter.send(request).await {
            self.transportation_contacts.remove(to_transporter);
            return Err(Error::TransporterDropped {
                symbol: to_transporter.to_string(),
                from: from_extractor.to_string(),
                to: to_transporter.to_string(),
            });
        }

        receiver.await.map_err(|_e| Error::TransporterDropped {
            symbol: to_transporter.to_string(),
            from: from_extractor.to_string(),
            to: to_transporter.to_string(),
        })?;
        Ok(())
    }
}

pub enum Error {
    TransporterDropped {
        symbol: String,
        from: String,
        to: String,
    },
    ExtractorDropped {
        symbol: String,
        from: String,
        to: String,
    },
}
