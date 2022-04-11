# Weather CLI
Simple client implementation for [WeatherServer](https://github.com/VladyslavYareschenko/weather_service_rpc). This tool allows you to get a weather forecast for a specified location and date. Communication with the server is done through the [weather_service_rpc](https://github.com/VladyslavYareschenko/weather_service_rpc).

Available commands are:
| Command | Description |
| ------ | ------ |
| get-providers | Returns the list of available weather providers from the server. |
| configure <provider> | Set the active provider to request the weather forecast. |
| get <address> <date> | Get the forecast for the specified location and date (in mm.dd.yyyy format). For example: `get London 01.22.2012`. |