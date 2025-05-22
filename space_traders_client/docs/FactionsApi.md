# \FactionsApi

All URIs are relative to *https://api.spacetraders.io/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_faction**](FactionsApi.md#get_faction) | **GET** /factions/{factionSymbol} | Faction details
[**get_factions**](FactionsApi.md#get_factions) | **GET** /factions | List factions
[**get_my_factions**](FactionsApi.md#get_my_factions) | **GET** /my/factions | Get My Factions



## get_faction

> models::GetFaction200Response get_faction(faction_symbol)
Faction details

View the details of a faction.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**faction_symbol** | **String** | The faction symbol | [required] |

### Return type

[**models::GetFaction200Response**](get_faction_200_response.md)

### Authorization

[AccountToken](../README.md#AccountToken), [AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_factions

> models::GetFactions200Response get_factions(page, limit)
List factions

Return a paginated list of all the factions in the game.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | What entry offset to request |  |[default to 1]
**limit** | Option<**i32**> | How many entries to return per page |  |[default to 10]

### Return type

[**models::GetFactions200Response**](get_factions_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_my_factions

> models::GetMyFactions200Response get_my_factions(page, limit)
Get My Factions

Retrieve factions with which the agent has reputation.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | What entry offset to request |  |[default to 1]
**limit** | Option<**i32**> | How many entries to return per page |  |[default to 10]

### Return type

[**models::GetMyFactions200Response**](get_my_factions_200_response.md)

### Authorization

[AccountToken](../README.md#AccountToken), [AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

