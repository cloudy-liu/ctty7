# Mermaid Diagram Test

This file tests Mermaid diagram rendering in the Markdown preview.

## Flowchart

```mermaid
graph TD
    A[Start] --> B{Is it working?}
    B -->|Yes| C[Great!]
    B -->|No| D[Debug]
    D --> B
    C --> E[End]
```

## Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant Editor
    participant Renderer
    
    User->>Editor: Open Markdown file
    Editor->>Renderer: Process Mermaid blocks
    Renderer->>Renderer: Render to SVG
    Renderer-->>Editor: Return processed content
    Editor-->>User: Display preview
```

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: File opened
    Processing --> Rendering: Mermaid detected
    Rendering --> Display: SVG generated
    Display --> Idle: Preview shown
    Display --> [*]: Window closed
```

## Class Diagram

```mermaid
classDiagram
    class MermaidConfig {
        +String primary_color
        +String background_color
        +String text_color
        +render() SVG
    }
    
    class MarkdownPreview {
        +process_blocks()
        +render()
    }
    
    MermaidConfig <-- MarkdownPreview: uses
```

## Regular Code (should not be processed)

```rust
fn main() {
    println!("This is regular code, not Mermaid");
}
```

## Mixed Content

Some text before the diagram.

```mermaid
graph LR
    A[Input] --> B[Process]
    B --> C[Output]
```

And some text after the diagram.
