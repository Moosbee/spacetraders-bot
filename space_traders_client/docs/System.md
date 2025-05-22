# System

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**constellation** | Option<**String**> | The constellation that the system is part of. | [optional]
**symbol** | **String** | The symbol of the system. | 
**sector_symbol** | **String** | The symbol of the sector. | 
**r#type** | [**models::SystemType**](SystemType.md) |  | 
**x** | **i32** | Relative position of the system in the sector in the x axis. | 
**y** | **i32** | Relative position of the system in the sector in the y axis. | 
**waypoints** | [**Vec<models::SystemWaypoint>**](SystemWaypoint.md) | Waypoints in this system. | 
**factions** | [**Vec<models::SystemFaction>**](SystemFaction.md) | Factions that control this system. | 
**name** | Option<**String**> | The name of the system. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


