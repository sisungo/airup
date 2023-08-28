# Module: `debug`
Module `debug` provides methods that are used for Airup debugging.

## Method: `debug.disconnect`
- Name: `debug.disconnect`
- Parameters: None
- Required Permissions: None
- Return Value: Never returns
- Description: Disconnects current IPC connection.

## Method: `debug.exit`
- Name: `debug.exit`
- Parameters: None
- Required Permissions: `power`
- Return Value: `null`
- Description: Makes `airupd` daemon process exit.

## Method: `debug.echo_raw`
- Name: `debug.echo_raw`
- Parameters: `Response` object
- Required Permissions: None
- Return Value: Content of the parameter.
- Description: Returns the `Response` object provided by the parameter.

## Method: `debug.reload_image`
- Name: `debug.reload_image`
- Parameters: None
- Required Permissions: `power`
- Return Value: `null`
- Description: Makes `airupd` daemon process reloads its process image.