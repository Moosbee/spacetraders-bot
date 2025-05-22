# \DataApi

All URIs are relative to *https://api.spacetraders.io/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_supply_chain**](DataApi.md#get_supply_chain) | **GET** /market/supply-chain | Describes trade relationships
[**websocket_departure_events**](DataApi.md#websocket_departure_events) | **GET** /my/socket.io | Subscribe to events



## get_supply_chain

> models::GetSupplyChain200Response get_supply_chain()
Describes trade relationships

Describes which import and exports map to each other.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetSupplyChain200Response**](get_supply_chain_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## websocket_departure_events

> websocket_departure_events()
Subscribe to events

Subscribe to departure events for a system.            ## WebSocket Events            The following events are available:            - `systems.{systemSymbol}.departure`: A ship has departed from the system.            ## Subscribe using a message with the following format:            ```json           {             \"action\": \"subscribe\",             \"systemSymbol\": \"{systemSymbol}\"           }           ```            ## Unsubscribe using a message with the following format:            ```json           {             \"action\": \"unsubscribe\",             \"systemSymbol\": \"{systemSymbol}\"           }           ```

### Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

