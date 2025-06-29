# \AccountsApi

All URIs are relative to *https://api.spacetraders.io/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_my_account**](AccountsApi.md#get_my_account) | **GET** /my/account | Get Account
[**register**](AccountsApi.md#register) | **POST** /register | Register New Agent



## get_my_account

> models::GetMyAccount200Response get_my_account()
Get Account

Fetch your account details.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::GetMyAccount200Response**](get_my_account_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## register

> models::Register201Response register(register_request)
Register New Agent

Creates a new agent and ties it to an account.  The agent symbol must consist of a 3-14 character string, and will be used to represent your agent. This symbol will prefix the symbol of every ship you own. Agent symbols will be cast to all uppercase characters.  This new agent will be tied to a starting faction of your choice, which determines your starting location, and will be granted an authorization token, a contract with their starting faction, a command ship that can fly across space with advanced capabilities, a small probe ship that can be used for reconnaissance, and 175,000 credits.  > #### Keep your token safe and secure > > Keep careful track of where you store your token. You can generate a new token from our account dashboard, but if someone else gains access to your token they will be able to use it to make API requests on your behalf until the end of the reset.  If you are new to SpaceTraders, It is recommended to register with the COSMIC faction, a faction that is well connected to the rest of the universe. After registering, you should try our interactive [quickstart guide](https://docs.spacetraders.io/quickstart/new-game) which will walk you through a few basic API requests in just a few minutes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**register_request** | [**RegisterRequest**](RegisterRequest.md) |  | [required] |

### Return type

[**models::Register201Response**](register_201_response.md)

### Authorization

[AccountToken](../README.md#AccountToken)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

