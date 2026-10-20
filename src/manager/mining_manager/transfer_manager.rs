use dashmap::DashMap;
use space_traders_client::models;
use space_traders_client::ShipSymbol;
use tokio::sync::mpsc;
use utils::safely_get_map;

#[derive(Debug)]
pub struct TransportTransferRequest {
    pub from_symbol: ShipSymbol,
    pub to_symbol: ShipSymbol,
    pub amount: i32,
    pub trade_symbol: models::TradeSymbol,
    pub extractor_contact: tokio::sync::mpsc::Sender<ExtractorTransferRequest>,
    pub callback: tokio::sync::oneshot::Sender<()>,
}

#[derive(Debug)]
pub struct ExtractorTransferRequest {
    pub from_symbol: ShipSymbol,
    pub to_symbol: ShipSymbol,
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
    extraction_contacts: DashMap<ShipSymbol, mpsc::Sender<ExtractorTransferRequest>>,
    transportation_contacts: DashMap<ShipSymbol, mpsc::Sender<TransportTransferRequest>>,
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
        symbol: &ShipSymbol,
        sender: mpsc::Sender<ExtractorTransferRequest>,
    ) {
        self.extraction_contacts.insert(symbol.clone(), sender);
    }

    pub fn add_transportation_contact(
        &self,
        symbol: &ShipSymbol,
        sender: mpsc::Sender<TransportTransferRequest>,
    ) {
        tracing::debug!(symbol = %symbol, "Adding transportation contact");
        self.transportation_contacts
            .insert(symbol.clone(), sender);
    }

    pub fn viable(&self, from_extractor: &ShipSymbol, to_transporter: &ShipSymbol) -> bool {
        let extractor = self.valid_extractor(from_extractor);
        let transporter = self.valid_transporter(to_transporter);

        extractor && transporter
    }

    pub fn valid_extractor(&self, symbol: &ShipSymbol) -> bool {
        {
            self.extraction_contacts
                .remove_if(symbol, |_, c| c.is_closed());
        }
        let refi = safely_get_map(&self.extraction_contacts, symbol);

        let is = match refi.as_ref() {
            Some(contact) => !contact.is_closed(),
            None => false,
        };

        tracing::debug!(symbol = %symbol, valid = %is, has_contact = %refi.is_some(), is_closed = ?refi.as_ref().map(|f| f.is_closed()), "Valid extractor");

        is
    }

    pub fn valid_transporter(&self, symbol: &ShipSymbol) -> bool {
        {
            self.transportation_contacts
                .remove_if(symbol, |_, c| c.is_closed());
        }
        let refi = safely_get_map(&self.transportation_contacts, symbol);

        let is = match refi.as_ref() {
            Some(contact) => !contact.is_closed(),
            None => false,
        };

        tracing::debug!(symbol = %symbol, valid = %is, has_contact = %refi.is_some(), is_closed = ?refi.as_ref().map(|f| f.is_closed()), "Valid transporter");

        is
    }

    pub async fn process_transfer(
        &self,
        from_extractor: &ShipSymbol,
        to_transporter: &ShipSymbol,
        symbol: models::TradeSymbol,
        amount: i32,
    ) -> Result<(), Error> {
        let transporter = safely_get_map(&self.transportation_contacts, to_transporter)
            .filter(|c| !c.is_closed())
            .ok_or(Error::TransporterDropped {
                symbol: to_transporter.clone(),
                from: from_extractor.clone(),
                to: to_transporter.clone(),
            })?;

        let extractor = safely_get_map(&self.extraction_contacts, from_extractor)
            .filter(|c| !c.is_closed())
            .ok_or(Error::ExtractorDropped {
                symbol: from_extractor.clone(),
                from: from_extractor.clone(),
                to: to_transporter.clone(),
            })?
            .clone();

        let (callback, receiver) = tokio::sync::oneshot::channel();
        let request = TransportTransferRequest {
            from_symbol: from_extractor.clone(),
            to_symbol: to_transporter.clone(),
            amount,
            trade_symbol: symbol,
            extractor_contact: extractor,
            callback,
        };

        if let Err(_err) = transporter.send(request).await {
            self.transportation_contacts.remove(to_transporter);
            return Err(Error::TransporterDropped {
                symbol: to_transporter.clone(),
                from: from_extractor.clone(),
                to: to_transporter.clone(),
            });
        }

        receiver.await.map_err(|_e| Error::TransporterDropped {
            symbol: to_transporter.clone(),
            from: from_extractor.clone(),
            to: to_transporter.clone(),
        })?;
        Ok(())
    }
}

pub enum Error {
    TransporterDropped {
        symbol: ShipSymbol,
        from: ShipSymbol,
        to: ShipSymbol,
    },
    ExtractorDropped {
        symbol: ShipSymbol,
        from: ShipSymbol,
        to: ShipSymbol,
    },
}
