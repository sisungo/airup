# Module: `system`

Module `system` provides methods for managing the system.

## Object: `Event`

**Name**: `Event`

**Fields**:
 - `id`: ID of this event.
 - `payload`: Payload data provided by this event.

## Method: `system.refresh`

**Name**: `system.refresh`

**Parameters**: None

**Return Value**: `[(string, error)] (array of a tuple which combines error position and error)`

**Description**: Refreshes some of `airupd`'s internal status.

## Method: `system.gc`

**Name**: `system.gc`

**Parameters**: None

**Return Value**: `null`

**Description**: Releases `airupd`'s cached system resources.

## Method: `system.query_service`

**Name**: `system.query_service`

**Parameters**: `string (name of service to query)` (optional)

**Return Value**: `QueryService` object

**Description**: Returns queried information of the service.

## Method: `system.query_system`

**Name**: `system.query_system`

**Parameters**: None

**Return Value**: `QuerySystem` object

**Description**: Returns queried macro information about the whole system.

## Method: `system.list_services`

**Name**: `system.list_services`

**Parameters**: None

**Return Value**: `string` array

**Description**: Returns a list over names of all installed services on the system.

## Method: `system.start_service`

**Name**: `system.start_service`

**Parameters**: `string (name of service to operate)`

**Return Value**: `null`

**Description**: Starts the specified service.

## Method: `system.stop_service`

**Name**: `system.stop_service`

**Parameters**: `string (name of service to operate)`

**Return Value**: `null`

**Description**: Stops the specified service.

## Method: `system.cache_service`

**Name**: `system.cache_service`

**Parameters**: `string (name of service to operate)`

**Return Value**: `null`

**Description**: Caches the specified service.

## Method: `system.uncache_service`

**Name**: `system.uncache_service`

**Parameters**: `string (name of service to operate)`

**Return Value**: `null`

**Description**: Uncaches the specified service.

## Method: `system.sideload_service`

**Name**: `system.sideload_service`

**Parameters**: `string (name of service)` and `Service` object

**Return Value**: `null`

**Description**: Caches the given service in given name.

## Method: `system.kill_service`

**Name**: `system.kill_service`

**Parameters**: `string (name of service to operate)`

**Return Value**: `null`

**Description**: Forces the specified service to stop.

## Method: `system.reload_service`

**Name**: `system.reload_service`

**Parameters**: `string (name of service to operate)`

**Return Value**: `null`

**Description**: Notifies the specified service to reload.

## Method: `system.trigger_event`

**Name**: `system.trigger_event`

**Parameters**: `Event` object

**Return Value**: `null`

**Description**: Triggers the specified event.

## Method: `system.enter_milestone`

**Name**: `system.enter_milestone`

**Parameters**: `string`

**Return Value**: `null`

**Description**: Enters a milestone.

## Method: `system.set_instance_name`

**Name**: `system.set_instance_name`

**Parameters**: `string`

**Return Value**: `null`

**Description**: Sets the server's instance name. If the string parameter was empty, it restores the default instance name.

## Method: `system.load_extension`

**Name**: `system.load_extension`

**Parameters**: `string (name of extension)`, `string (path of the extension's socket)`

**Return Value**: `null`

**Description**: Loads an Airup extension.

## Method: `system.unload_extension`

**Name**: `system.unload_extension`

**Parameters**: `string (name of extension)`

**Return Value**: `null`

**Description**: Unloads an Airup extension.

**Possible Errors**:

 - `NOT_FOUND`: The specified extension was not installed yet.
