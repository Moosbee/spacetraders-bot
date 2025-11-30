-- Add up migration script here
CREATE TABLE ExportImportMapping (
  export_symbol trade_symbol NOT NULL,
  import_symbol trade_symbol NOT NULL,
  PRIMARY KEY (export_symbol, import_symbol)
);