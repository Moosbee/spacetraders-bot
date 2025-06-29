# Survey

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**signature** | **String** | A unique signature for the location of this survey. This signature is verified when attempting an extraction using this survey. | 
**symbol** | **String** | The symbol of the waypoint that this survey is for. | 
**deposits** | [**Vec<models::SurveyDeposit>**](SurveyDeposit.md) | A list of deposits that can be found at this location. A ship will extract one of these deposits when using this survey in an extraction request. If multiple deposits of the same type are present, the chance of extracting that deposit is increased. | 
**expiration** | **String** | The date and time when the survey expires. After this date and time, the survey will no longer be available for extraction. | 
**size** | [**models::SurveySize**](SurveySize.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


