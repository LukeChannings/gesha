# Documentation for Gesha

<a name="documentation-for-api-endpoints"></a>
## Documentation for API Endpoints

All URIs are relative to *http://localhost/api*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*DefaultApi* | [**getConfig**](Apis/DefaultApi.md#getconfig) | **GET** /config | Your GET endpoint
*DefaultApi* | [**getPidEnabled**](Apis/DefaultApi.md#getpidenabled) | **GET** /pid/enabled | Your GET endpoint
*DefaultApi* | [**getPidOutput**](Apis/DefaultApi.md#getpidoutput) | **GET** /pid/output | Your GET endpoint
*DefaultApi* | [**getStreamPidOutput**](Apis/DefaultApi.md#getstreampidoutput) | **GET** /stream/pid/output | Your GET endpoint
*DefaultApi* | [**getStreamTempCurrent**](Apis/DefaultApi.md#getstreamtempcurrent) | **GET** /stream/temp/current | Your GET endpoint
*DefaultApi* | [**getTemp**](Apis/DefaultApi.md#gettemp) | **GET** /temp/current | Your GET endpoint
*DefaultApi* | [**getTempTarget**](Apis/DefaultApi.md#gettemptarget) | **GET** /temp/target | Your GET endpoint
*DefaultApi* | [**postConfig**](Apis/DefaultApi.md#postconfig) | **POST** /config | Update the running app configuration
*DefaultApi* | [**postPidEnabled**](Apis/DefaultApi.md#postpidenabled) | **POST** /pid/enabled | 
*DefaultApi* | [**postTempTarget**](Apis/DefaultApi.md#posttemptarget) | **POST** /temp/target | 


<a name="documentation-for-models"></a>
## Documentation for Models

 - [Config](.//Models/Config.md)
 - [OperationResult](.//Models/OperationResult.md)
 - [PidEnabled](.//Models/PidEnabled.md)
 - [PidOutput](.//Models/PidOutput.md)
 - [TemperatureItem](.//Models/TemperatureItem.md)
 - [TemperatureTarget](.//Models/TemperatureTarget.md)


<a name="documentation-for-authorization"></a>
## Documentation for Authorization

All endpoints do not require authorization.
