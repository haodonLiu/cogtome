# Text Processing Assembly

Transform text content using various operations.

## Usage

```json
{
  "text": "hello world",
  "operation": "uppercase"
}
```

## Supported Operations

- `uppercase` - Convert text to uppercase
- `reverse` - Reverse the text string

## Examples

**Input:**
```json
{
  "text": "hello",
  "operation": "uppercase"
}
```

**Output:**
```json
{
  "result": "HELLO"
}
```

## Notes

- The operation parameter determines which transformation is applied
- Currently uses text-uppercase unit directly
- Future versions will support dynamic operation selection via match node
