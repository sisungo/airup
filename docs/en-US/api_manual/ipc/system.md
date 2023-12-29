# Module: `system`
Module `system` provides methods for managing the system.

## Method: `system.refresh`
- Name: `system.refresh`
- Parameters: None
- Return Value: `null`
- Description: Refreshes some of `airupd`'s internal status.

## Method: `system.gc`
- Name: `system.gc`
- Parameters: None
- Return Value: `null`
- Description: Releases `airupd`'s cached system resources.

## Method: `system.query_service`
- Name: `system.query_service`
- Parameters: `string (name of service to query)` (optional)
- Return Value: `QueryService` object
- Description: Returns queried information of the service.

## Method: `system.query_system`
- Name: `system.query_system`
- Parameters: None
- Return Value: `QuerySystem` object
- Description: Returns queried macro information about the whole system.

## Method: `system.start_service`
- Name: `system.start_service`
- Parameters: `string (name of service to operate)`
- Return Value: `null`
- Description: Starts the specified service.

## Method: `system.stop_service`
- Name: `system.stop_service`
- Parameters: `string (name of service to operate)`
- Return Value: `null`
- Description: Stops the specified service.

## Method: `system.kill_service`
- Name: `system.kill_service`
- Parameters: `string (name of service to operate)`
- Return Value: `null`
- Description: Forces the specified service to stop.

## Method: `system.reload_service`
- Name: `system.reload_service`
- Parameters: `string (name of service to operate)`
- Return Value: `null`
- Description: Notifies the specified service to reload.