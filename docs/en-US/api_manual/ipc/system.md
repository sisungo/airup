# Module: `system`
Module `system` provides methods for managing the system.

## Method: `system.refresh`
- Name: `system.refresh`
- Parameters: None
- Required Permissions: `refresh`
- Return Value: `null`
- Description: Refreshes `airupd`'s cached system resources and internal status.

## Method: `system.query_service`
- Name: `system.query_service`
- Parameters: `string (name of service to query)` (optional)
- Required Permissions: `query_services`
- Return Value: `QueryResult` object OR `string array`
- Description: If the parameter is `null`, returns a list of names of all loaded services. If not, returns queried status of the service.

## Method: `system.start_service`
- Name: `system.start_service`
- Parameters: `string (name of service to operate)`
- Required Permissions: `manage_services`
- Return Value: `null`
- Description: Starts the specified service.

## Method: `system.stop_service`
- Name: `system.stop_service`
- Parameters: `string (name of service to operate)`
- Required Permissions: `manage_services`
- Return Value: `null`
- Description: Stops the specified service.