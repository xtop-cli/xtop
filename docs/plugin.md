<h1>Plugin System</h1>

<p>xtop hosts <strong>plugins</strong>: extra functionality shipped as separate
crates that implement the contract in <code>xtop-plugin-api</code> (a crate of
the <code>xtop-cli/api</code> repo). Plugins are compile-time: the kernel wires
them through Cargo git dependencies and feature flags, never through runtime
discovery. The built-in plugin is <code>xtop-plugin-samurai</code>
(see <a href="multi-repo.md">multi-repo.md</a> for the ecosystem layout).</p>

<p>You can build xtop without any plugin or extension:</p>

<pre><code>cargo build --release --no-default-features</code></pre>

<p>Or with the default set (samurai plugin + MCP extension):</p>

<pre><code>cargo build --release</code></pre>

<hr>

<h2 id="architecture">Architecture</h2>

<p>The plugin system has four kernel components:</p>

<table>
  <thead>
    <tr><th>Component</th><th>Kernel location</th><th>Purpose</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>Plugin</code> trait + contract types</td>
      <td><code>xtop_plugin_api</code> (crate, external)</td>
      <td>Interface every plugin implements; manifest, capabilities, errors, data model</td>
    </tr>
    <tr>
      <td><code>PluginManager</code></td>
      <td><code>src/plugins/manager.rs</code></td>
      <td>Registers, ticks and dispatches events to plugins</td>
    </tr>
    <tr>
      <td><code>HostState</code> impl</td>
      <td><code>src/plugins/host.rs</code></td>
      <td>Kernel-side view of the live state plugins may touch</td>
    </tr>
    <tr>
      <td><code>CompositeProvider</code></td>
      <td><code>src/providers/composite.rs</code></td>
      <td>Merges the kernel provider with plugin data providers</td>
    </tr>
  </tbody>
</table>

<p>Plugins never depend on the kernel: they see state only through
<code>PluginContext</code>, which is built over the <code>HostState</code> trait.
Plugin widgets render over <code>&amp;dyn HostState</code> through
<code>xtop_plugin_api::PluginWidget</code> (distinct from the widget-pack
registration in <code>xtop-widget-api</code>, which draws over
<code>WidgetState</code>).</p>

<hr>

<h2 id="the-plugin-trait">The <code>Plugin</code> Trait</h2>

<pre><code>pub trait Plugin: Debug + Send {
    fn manifest(&amp;self) -&gt; PluginManifest;
    fn on_enable(&amp;mut self, ctx: &amp;mut PluginContext) -&gt; Result&lt;(), PluginError&gt;;
    fn on_disable(&amp;mut self, ctx: &amp;mut PluginContext) -&gt; Result&lt;(), PluginError&gt;;
    fn on_tick(&amp;mut self, ctx: &amp;mut PluginContext) -&gt; Result&lt;(), PluginError&gt;;
    fn on_key(&amp;mut self, ctx: &amp;mut PluginContext, key: &amp;str) -&gt; Result&lt;bool, PluginError&gt;;
    fn data_provider(&amp;self) -&gt; Option&lt;Box&lt;dyn SystemDataProvider&gt;&gt;;
    fn widget(&amp;self) -&gt; Option&lt;PluginWidget&gt;;
    fn execute(&amp;mut self, ctx: &amp;mut PluginContext, action: &amp;str, params: &amp;str)
        -&gt; Result&lt;String, PluginError&gt;;
}</code></pre>

<p>All methods except <code>manifest()</code> have default implementations, so a
minimal plugin only declares its manifest.</p>

<table>
  <thead>
    <tr><th>Method</th><th>Default</th><th>Called When</th></tr>
  </thead>
  <tbody>
    <tr><td><code>manifest()</code></td><td>required</td><td>Any time metadata is needed</td></tr>
    <tr><td><code>on_enable()</code></td><td>no-op</td><td>Plugin is registered at startup</td></tr>
    <tr><td><code>on_disable()</code></td><td>no-op</td><td>xtop shuts down</td></tr>
    <tr><td><code>on_tick()</code></td><td>no-op</td><td>Every update cycle (~1s by default)</td></tr>
    <tr><td><code>on_key()</code></td><td><code>false</code></td><td>Key press in Normal mode (returns <code>true</code> to consume)</td></tr>
    <tr><td><code>data_provider()</code></td><td><code>None</code></td><td>Startup (merged into the CompositeProvider)</td></tr>
    <tr><td><code>widget()</code></td><td><code>None</code></td><td>After every tick (refreshes the plugin widget map)</td></tr>
    <tr><td><code>execute()</code></td><td><code>UnknownAction</code></td><td>External agent (AI, CLI, MCP) invokes a command</td></tr>
  </tbody>
</table>

<h3 id="pluginmanifest">PluginManifest</h3>

<p>Each plugin declares its identity and needs in
<code>manifest().capabilities</code>:</p>

<ul>
  <li><code>ReadSystemInfo</code> -- read system metrics</li>
  <li><code>KillProcesses</code> -- terminate processes</li>
  <li><code>ModifyConfig</code> -- change themes, layouts, thresholds, intervals</li>
  <li><code>RenderWidgets</code> -- register custom TUI widgets</li>
  <li><code>Custom(String)</code> -- anything not covered above</li>
</ul>

<h3 id="plugincontext">PluginContext</h3>

<p>Safe, limited access to application state. Capability-gated reads return
<code>Result</code>; writes require the matching capability and fail with a
<code>PluginError::Recoverable</code> otherwise:</p>

<pre><code>ctx.snapshot()?                 // Full SystemSnapshot (needs ReadSystemInfo)
ctx.top_processes(n)?           // Top n processes by CPU, sorted desc
ctx.system_info()?              // Hostname, OS, kernel (needs ReadSystemInfo)
ctx.kill_process(pid)?          // Kill process by PID (needs KillProcesses)
ctx.set_alert_thresholds(cpu, mem, disk)?  // (needs ModifyConfig)
ctx.set_theme_by_name("tokio")? // (needs ModifyConfig)
ctx.set_layout_by_name("CPU Focus")?       // (needs ModifyConfig)
ctx.set_update_interval(500)?   // (needs ModifyConfig)
ctx.alerts()                    // Current alert thresholds
ctx.config()                    // Theme, layout, interval, hostname
ctx.data_dir()                  // Host-provided plugin data dir</code></pre>

<hr>

<h2 id="data-providers">Data Providers</h2>

<p>The kernel samples the system with its own provider
(<code>SysinfoProvider</code> in <code>src/providers/sysinfo/</code>) and composes
plugin providers through <code>CompositeProvider</code>:</p>

<ul>
  <li><code>refresh_all()</code> refreshes the primary provider and every extra</li>
  <li><code>snapshot()</code> delegates to the primary</li>
  <li><code>disk_io()</code>, <code>batteries()</code>, <code>gpu_info()</code> and
      <code>system_info()</code> use the primary result when non-empty, otherwise
      the first non-empty extra provider result</li>
  <li><code>kill_process()</code> tries the primary first, then the extras</li>
</ul>

<hr>

<h2 id="widget-registration">Plugin Widgets</h2>

<p>A plugin can register one custom widget via <code>widget()</code>. The render
closure receives the plugin view of the state (<code>&amp;dyn HostState</code>)
and draws with plain ratatui:</p>

<pre><code>fn widget(&amp;self) -&gt; Option&lt;PluginWidget&gt; {
    Some(PluginWidget {
        name: "samurai".to_string(),
        render: Arc::new(|f, state, area| {
            // Draw using ratatui; `state` is &amp;dyn HostState.
        }),
    })
}</code></pre>

<p>Custom widgets are placed in layouts by name in JSONC layout files:</p>

<pre><code>{
    "name": "my-layout",
    "root": {
        "direction": "vertical",
        "areas": [
            { "widget": "header", "size": 3 },
            { "widget": "samurai", "size": "30%" },
            { "widget": "processes", "size": "*" }
        ]
    }
}</code></pre>

<p>Plugin widgets take precedence over widget-pack widgets with the same name
(the kernel resolves plugin widgets first at render time). A layout
referencing a name no pack and no plugin provides prints a one-time warning
to stderr.</p>

<hr>

<h2 id="cli-commands">CLI Commands</h2>

<table>
  <thead>
    <tr><th>Command</th><th>Description</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>xtop plugin list</code></td>
      <td>List plugins wired into the kernel <code>Cargo.toml</code> (feature entries with <code>dep:&lt;name&gt;</code>)</td>
    </tr>
    <tr>
      <td><code>xtop plugin install &lt;name&gt;</code></td>
      <td>Install a plugin by name from <code>github.com/xtop-cli/plugins</code></td>
    </tr>
    <tr>
      <td><code>xtop plugin install &lt;url&gt;</code></td>
      <td>Install a plugin from any git URL</td>
    </tr>
    <tr>
      <td><code>xtop plugin scaffold &lt;name&gt;</code></td>
      <td>Create a new plugin crate template in <code>plugins-dev/</code> (git-ignored)</td>
    </tr>
  </tbody>
</table>

<h3 id="install-flow">Install Flow (real behavior)</h3>

<p><code>xtop plugin install</code> does <strong>not</strong> download a binary and does
<strong>not</strong> register anything at runtime. It edits the kernel's own
<code>Cargo.toml</code> (a self-modifying-source workflow):</p>

<ol>
  <li>Resolves the source repo: <code>github.com/xtop-cli/plugins</code> for a name, or the given URL</li>
  <li>Clones it (shallow, sparse) into a temp dir</li>
  <li>Locates the <code>xtop-plugin-&lt;name&gt;</code> crate inside the clone
      (repo root, or <code>plugins/</code>/<code>crates/</code> subfolders)</li>
  <li>Adds an optional git dependency + a feature flag to the kernel's root
      <code>Cargo.toml</code> (the same pattern the built-in
      <code>xtop-plugin-samurai</code> uses)</li>
  <li>Runs <code>cargo check</code> to verify the manifest resolves, then cleans up</li>
</ol>

<p>The plugin is registered but <strong>not enabled by default</strong>. To enable
it, add its feature to the <code>default</code> list in <code>[features]</code> in the
kernel <code>Cargo.toml</code> (or build with <code>--features &lt;name&gt;</code>) and
recompile. Every installed plugin also needs a registration line in
<code>src/commands/share/bootstrap.rs</code> under its feature flag.</p>

<pre><code># Build xtop with samurai plugin enabled (default already includes it)
cargo build --release --features plugin-samurai

# Build xtop with samurai + another plugin
cargo build --release --features "plugin-samurai,plugin-mything"</code></pre>

<hr>

<h2 id="extensions-mcp">Extensions: the MCP Server</h2>

<p>Extensions are the kernel's server-style hooks (contract:
<code>xtop-extension-api</code>). The shipped extension is the MCP
(Model Context Protocol) server in the <code>xtop-cli/extensions</code> repo
(<code>xtop-extension-mcp</code>), which exposes the hosted plugins' actions as
MCP tools over stdio. It is compiled in when the <code>mcp-extension</code>
feature is on (part of the default features).</p>

<pre><code>xtop mcp</code></pre>

<p>The MCP tools are executed against the samurai plugin, so the
<code>plugin-samurai</code> feature is required too. Compatible clients include
Claude Desktop, Cline, Cursor, and Continue.dev.</p>

<h3>Claude Desktop configuration</h3>

<pre><code>{
  "mcpServers": {
    "xtop": {
      "command": "xtop",
      "args": ["mcp"]
    }
  }
}</code></pre>

<h3>Interactive testing</h3>

<pre><code># Single command
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"system_summary","arguments":{}}}' | xtop mcp

# Interactive session
xtop mcp</code></pre>

<p>The MCP extension depends on <code>xtop-plugin-samurai</code> at compile time:
the plugin id and the 12 action names are single-sourced constants
(<code>PLUGIN_ID</code>, <code>actions::*</code>), so the tool table can never drift
from the plugin implementation.</p>

<hr>

<h2 id="runtime-widgets">Runtime Widgets: WASM and External Processes</h2>

<p>Plugins and widget packs are compile-time. For code that should not be
compiled into the kernel, xtop ships two optional hosts (both off by
default):</p>

<ul>
  <li><code>plugin-wasm</code> — loads sandboxed <code>.wasm</code> widgets
      with wasmi (fuel and memory limits, hot reload).</li>
  <li><code>plugin-external</code> — runs one helper process per widget over
      line-delimited JSON, so Lua, Python, Node or any language with a
      runtime can render a widget.</li>
</ul>

<p>Runtime widgets register through the same plugin widget path as compiled
plugins, keep precedence over packs, and are referenced in layouts by the
name in their manifest. Build with
<code>--features plugin-wasm,plugin-external</code> and see
<a href="customization.md#runtime-widgets">customization.md
(&ldquo;Runtime Widgets&rdquo;)</a> for directories, examples and the demo.</p>

<hr>

<h2 id="adding-a-plugin-manually">Adding a Plugin Manually</h2>

<ol>
  <li>
    <p>Scaffold the crate (or write it by hand in your own repo):</p>
    <pre><code>xtop plugin scaffold mything   # creates plugins-dev/xtop-plugin-mything/</code></pre>
  </li>
  <li>
    <p>Make it a git repo and push it (the crate must live at the repo root or under
    <code>plugins/</code>/<code>crates/</code>, e.g. <code>github.com/you/xtop-plugin-mything</code>).</p>
  </li>
  <li>
    <p>Install it into the kernel (adds an optional git dependency + feature flag in the root <code>Cargo.toml</code>):</p>
    <pre><code>xtop plugin install https://github.com/you/xtop-plugin-mything</code></pre>
    <p>Equivalent manual edit:</p>
    <pre><code>[dependencies]
xtop-plugin-mything = { git = "https://github.com/you/xtop-plugin-mything", optional = true }

[features]
plugin-mything = ["dep:xtop-plugin-mything"]</code></pre>
  </li>
  <li>
    <p>Register it behind the feature flag in <code>src/commands/share/bootstrap.rs</code>:</p>
    <pre><code>#[cfg(feature = "plugin-mything")]
use xtop_plugin_mything::MythingPlugin;

// In register_plugins():
#[cfg(feature = "plugin-mything")]
{
    let plugin = Box::new(MythingPlugin::new());
    if let Err(e) = mgr.register(plugin, state) {
        eprintln!("[xtop] failed to load mything plugin: {e}");
    }
}</code></pre>
  </li>
</ol>

<p>Contract details for plugin authors (manifest, capabilities, error types,
widgets) live in the api repo docs; the samurai plugin in
<code>xtop-cli/plugins</code> is the reference implementation.</p>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
