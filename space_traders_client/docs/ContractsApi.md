# \ContractsApi

All URIs are relative to *https://api.spacetraders.io/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**accept_contract**](ContractsApi.md#accept_contract) | **POST** /my/contracts/{contractId}/accept | Accept Contract
[**deliver_contract**](ContractsApi.md#deliver_contract) | **POST** /my/contracts/{contractId}/deliver | Deliver Cargo to Contract
[**fulfill_contract**](ContractsApi.md#fulfill_contract) | **POST** /my/contracts/{contractId}/fulfill | Fulfill Contract
[**get_contract**](ContractsApi.md#get_contract) | **GET** /my/contracts/{contractId} | Get Contract
[**get_contracts**](ContractsApi.md#get_contracts) | **GET** /my/contracts | List Contracts
[**negotiate_contract_0**](ContractsApi.md#negotiate_contract_0) | **POST** /my/ships/{shipSymbol}/negotiate/contract | Negotiate Contract



## accept_contract

> models::AcceptContract200Response accept_contract(contract_id)
Accept Contract

Accept a contract by ID.   You can only accept contracts that were offered to you, were not accepted yet, and whose deadlines has not passed yet.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**contract_id** | **String** | The contract ID to accept. | [required] |

### Return type

[**models::AcceptContract200Response**](accept_contract_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## deliver_contract

> models::DeliverContract200Response deliver_contract(contract_id, deliver_contract_request)
Deliver Cargo to Contract

Deliver cargo to a contract.  In order to use this API, a ship must be at the delivery location (denoted in the delivery terms as `destinationSymbol` of a contract) and must have a number of units of a good required by this contract in its cargo.  Cargo that was delivered will be removed from the ship's cargo.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**contract_id** | **String** | The ID of the contract. | [required] |
**deliver_contract_request** | [**DeliverContractRequest**](DeliverContractRequest.md) |  | [required] |

### Return type

[**models::DeliverContract200Response**](deliver_contract_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fulfill_contract

> models::FulfillContract200Response fulfill_contract(contract_id)
Fulfill Contract

Fulfill a contract. Can only be used on contracts that have all of their delivery terms fulfilled.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**contract_id** | **String** | The ID of the contract to fulfill. | [required] |

### Return type

[**models::FulfillContract200Response**](fulfill_contract_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_contract

> models::GetContract200Response get_contract(contract_id)
Get Contract

Get the details of a specific contract.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**contract_id** | **String** | The contract ID to accept. | [required] |

### Return type

[**models::GetContract200Response**](get_contract_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_contracts

> models::GetContracts200Response get_contracts(page, limit)
List Contracts

Return a paginated list of all your contracts.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**page** | Option<**i32**> | What entry offset to request |  |[default to 1]
**limit** | Option<**i32**> | How many entries to return per page |  |[default to 10]

### Return type

[**models::GetContracts200Response**](get_contracts_200_response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## negotiate_contract_0

> models::NegotiateContract201Response negotiate_contract_0(ship_symbol)
Negotiate Contract

Negotiate a new contract with the HQ.  In order to negotiate a new contract, an agent must not have ongoing or offered contracts over the allowed maximum amount. Currently the maximum contracts an agent can have at a time is 1.  Once a contract is negotiated, it is added to the list of contracts offered to the agent, which the agent can then accept.   The ship must be present at any waypoint with a faction present to negotiate a contract with that faction.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**ship_symbol** | **String** | The symbol of the ship. | [required] |

### Return type

[**models::NegotiateContract201Response**](Negotiate_Contract_201_Response.md)

### Authorization

[AgentToken](../README.md#AgentToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

