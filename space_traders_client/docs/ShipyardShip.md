# ShipyardShip

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | [**models::ShipType**](ShipType.md) |  | 
**name** | **String** | Name of the ship. | 
**description** | **String** | Description of the ship. | 
**activity** | Option<[**models::ActivityLevel**](ActivityLevel.md)> |  | [optional]
**supply** | [**models::SupplyLevel**](SupplyLevel.md) |  | 
**purchase_price** | **i32** | The purchase price of the ship. | 
**frame** | [**models::ShipFrame**](ShipFrame.md) |  | 
**reactor** | [**models::ShipReactor**](ShipReactor.md) |  | 
**engine** | [**models::ShipEngine**](ShipEngine.md) |  | 
**modules** | [**Vec<models::ShipModule>**](ShipModule.md) | Modules installed in this ship. | 
**mounts** | [**Vec<models::ShipMount>**](ShipMount.md) | Mounts installed in this ship. | 
**crew** | [**models::ShipyardShipCrew**](ShipyardShip_crew.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


