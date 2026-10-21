# HTML preview

## User-visible contract

Local Windows `.html` and `.htm` files open in embedded browser preview by
default and can switch to the existing source editor. Explicit line navigation
opens source, and reactivating an open file keeps its selected mode. Preserve
the original document URL, CSS, JavaScript, relative resources and page colors.
Compare against Chrome at the same viewport, zoom and resource-loading conditions.

The first version previews saved files, offers refresh and external browser
opening, and leaves unsaved edits intact. An unsaved buffer must ask the user
to save before previewing or refreshing, without silently saving it. Remote
Host files retain source editing and are outside this first version.

## Resource and platform boundaries

Create WebView2 only when preview is requested. Reuse at most one active HTML
preview per application window. Switching HTML files must not retain one
browser view per document. Closing preview releases the browser view; switching
to another application surface must not leave a native view covering it.

Use the gpui-wry package from the existing gpui-component revision where
possible. Do not enable HTML preview on unverified platforms. Missing runtime
or view creation errors must be visible and recoverable in the source editor.
Do not add a JavaScript bridge to terminal or filesystem operations. Application
dialogs and menus must remain usable above the preview.

Markdown preview remains native and retains existing editing, navigation,
selection and layout behavior. Do not change global Windows rendering settings
without demonstrating why the change is needed and measuring its impact.

## Acceptance evidence

Capture the same local fixture before and after implementation, plus a Chrome
reference where possible. Exercise CSS layout, relative CSS/image/script loads,
JavaScript interaction, refresh, source switching, resizing and cleanup.

Measure comparable builds on the same machine. Record idle baseline, HTML source,
active preview, repeated navigation and closed preview. Include the application's
WebView2 processes in process-group memory; report resident and private committed
memory distinctly. Record preview load timing and idle CPU. Publish actual
measurements, environment, methodology and unverified cases in the PR.
