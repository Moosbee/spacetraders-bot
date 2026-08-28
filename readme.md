




## Mining Manager visualization

```mermaid
sequenceDiagram
    participant Transporter
    participant MiningManager
    participant Extractor-Y
    par Either
      Extractor-Y ->> MiningManager: Extraction complete
      Transporter ->> MiningManager: Arrived at Wp
    end
    loop until all extractors are emty or Transporter is full
        MiningManager -> MiningManager: get non empty extractors
        MiningManager -> MiningManager: get non empty Transporter
        MiningManager ->> Transporter: Take X * TradeGood from Extractor-Y
        Transporter ->> Extractor-Y: Give me X * TradeGood
        Extractor-Y ->> Transporter: Update your Cargo because you recieved X * TradeGood
        Transporter ->> MiningManager: Transfer complete
    end
```

## Todo
- update budgeting system to include ship transfers
- rework Manuel control
- update mining transfer system
- speed up ShipProcurementManager
- connect transactions with fleets
- fix marketTransactionSummary fuel display
- rewrite database provider to properly use inversion of controle