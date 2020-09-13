# DefaultApi

All URIs are relative to *http://localhost/api*

Method | HTTP request | Description
------------- | ------------- | -------------
[**getConfig**](DefaultApi.md#getConfig) | **GET** /config | Your GET endpoint
[**getPidEnabled**](DefaultApi.md#getPidEnabled) | **GET** /pid/enabled | Your GET endpoint
[**getPidOutput**](DefaultApi.md#getPidOutput) | **GET** /pid/output | Your GET endpoint
[**getStateCurrent**](DefaultApi.md#getStateCurrent) | **GET** /state/current | 
[**getStateStream**](DefaultApi.md#getStateStream) | **GET** /stream/state | 
[**getStreamPidOutput**](DefaultApi.md#getStreamPidOutput) | **GET** /stream/pid/output | Your GET endpoint
[**getStreamTempCurrent**](DefaultApi.md#getStreamTempCurrent) | **GET** /stream/temp/current | Your GET endpoint
[**getTemp**](DefaultApi.md#getTemp) | **GET** /temp/current | Your GET endpoint
[**getTempTarget**](DefaultApi.md#getTempTarget) | **GET** /temp/target | Your GET endpoint
[**postConfig**](DefaultApi.md#postConfig) | **POST** /config | 
[**postPidEnabled**](DefaultApi.md#postPidEnabled) | **POST** /pid/enabled | 
[**postTempTarget**](DefaultApi.md#postTempTarget) | **POST** /temp/target | 


<a name="getConfig"></a>
# **getConfig**
> Config getConfig()

Your GET endpoint

    Get the running app configuration

### Parameters
This endpoint does not need any parameter.

### Return type

[**Config**](..//Models/Config.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getPidEnabled"></a>
# **getPidEnabled**
> PidEnabled getPidEnabled()

Your GET endpoint

### Parameters
This endpoint does not need any parameter.

### Return type

[**PidEnabled**](..//Models/PidEnabled.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getPidOutput"></a>
# **getPidOutput**
> PidOutput getPidOutput()

Your GET endpoint

### Parameters
This endpoint does not need any parameter.

### Return type

[**PidOutput**](..//Models/PidOutput.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getStateCurrent"></a>
# **getStateCurrent**
> State getStateCurrent()



### Parameters
This endpoint does not need any parameter.

### Return type

[**State**](..//Models/State.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getStateStream"></a>
# **getStateStream**
> State getStateStream()



### Parameters
This endpoint does not need any parameter.

### Return type

[**State**](..//Models/State.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/event-stream

<a name="getStreamPidOutput"></a>
# **getStreamPidOutput**
> PidOutput getStreamPidOutput()

Your GET endpoint

### Parameters
This endpoint does not need any parameter.

### Return type

[**PidOutput**](..//Models/PidOutput.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/event-stream

<a name="getStreamTempCurrent"></a>
# **getStreamTempCurrent**
> TemperatureItem getStreamTempCurrent(unit, sampleRateMs)

Your GET endpoint

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **unit** | **String**|  | [default to null] [enum: c, f]
 **sampleRateMs** | **BigDecimal**|  | [optional] [default to null]

### Return type

[**TemperatureItem**](..//Models/TemperatureItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/event-stream

<a name="getTemp"></a>
# **getTemp**
> TemperatureItem getTemp(unit)

Your GET endpoint

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **unit** | **String**|  | [default to null] [enum: c, f]

### Return type

[**TemperatureItem**](..//Models/TemperatureItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getTempTarget"></a>
# **getTempTarget**
> TemperatureTarget getTempTarget()

Your GET endpoint

### Parameters
This endpoint does not need any parameter.

### Return type

[**TemperatureTarget**](..//Models/TemperatureTarget.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="postConfig"></a>
# **postConfig**
> OperationResult postConfig(config)



    Update the running app configuration

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **config** | [**Config**](..//Models/Config.md)|  | [optional]

### Return type

[**OperationResult**](..//Models/OperationResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="postPidEnabled"></a>
# **postPidEnabled**
> OperationResult postPidEnabled(pidEnabled)



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **pidEnabled** | [**PidEnabled**](..//Models/PidEnabled.md)|  | [optional]

### Return type

[**OperationResult**](..//Models/OperationResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="postTempTarget"></a>
# **postTempTarget**
> OperationResult postTempTarget(temperatureTarget)



### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **temperatureTarget** | [**TemperatureTarget**](..//Models/TemperatureTarget.md)|  | [optional]

### Return type

[**OperationResult**](..//Models/OperationResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

