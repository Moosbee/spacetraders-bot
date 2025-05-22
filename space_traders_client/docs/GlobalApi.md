# \GlobalApi

All URIs are relative to *https://api.spacetraders.io/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_error_codes**](GlobalApi.md#get_error_codes) | **GET** /error-codes | Error code list
[**get_status**](GlobalApi.md#get_status) | **GET** / | Server status



## get_error_codes

> models::GetErrorCodes200Response get_error_codes()
Error code list

Return a list of all possible error codes thrown by the game server.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetErrorCodes200Response**](get_error_codes_200_response.md)

### Authorization

[AccountToken](../README.md#AccountToken), [AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_status

> models::GetStatus200Response get_status()
Server status

Return the status of the game server. This also includes a few global elements, such as announcements, server reset dates and leaderboards.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetStatus200Response**](get_status_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

