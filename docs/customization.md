<h1 id="customization-guide">Customization Guide</h1>

<p>xtop supports runtime customization of color themes and layout modes via external JSONC files. This guide explains how to create and manage your own themes and layouts.</p>

<hr>

<h2 id="table-of-contents">Table of Contents</h2>

<ul>
  <li><a href="#themes">Themes</a>
    <ul>
      <li><a href="#theme-location">Location</a></li>
      <li><a href="#theme-format">Format</a></li>
      <li><a href="#palette-reference">Palette Reference</a></li>
      <li><a href="#starter-themes">Starter Themes</a></li>
      <li><a href="#loading-order">Loading Order</a></li>
      <li><a href="#theme-tips">Tips</a></li>
    </ul>
  </li>
  <li><a href="#layouts">Layouts</a>
    <ul>
      <li><a href="#layout-location">Location</a></li>
      <li><a href="#layout-format">Format</a></li>
      <li><a href="#size-constraints">Size Constraints</a></li>
      <li><a href="#available-widgets">Available Widgets</a></li>
      <li><a href="#layout-examples">Examples</a></li>
      <li><a href="#starter-layouts">Starter Layouts</a></li>
      <li><a href="#cycling-order">Cycling Order</a></li>
      <li><a href="#layout-notes">Notes</a></li>
    </ul>
  </li>
  <li><a href="#widget-packs">Widget Packs</a></li>
  <li><a href="#runtime-widgets">Runtime Widgets (WASM / External Processes)</a></li>
</ul>

<hr>

<h2 id="themes">Themes</h2>

<h3 id="theme-location">Location</h3>

<p>Theme files live in the <code>themes/</code> subfolder of the platform config
directory (the same tree as <code>config.json</code> and the layouts):</p>

<pre><code>~/.config/xtop/themes/*.jsonc                (Linux)
~/Library/Application Support/xtop/themes/   (macOS)
%APPDATA%\xtop\themes\                       (Windows)
</code></pre>

<p>The directory and the shipped theme files are created automatically on
first run; no manual copy is needed.</p>

<h3 id="theme-format">Format</h3>

<p>Each theme file defines a <code>name</code>, an explicit
<code>background</code>/<code>foreground</code> pair and a 16-entry
<code>palette</code> (format v2, UX8.1). Colors are hex strings with an
optional <code>#</code> prefix. Comments (<code>//</code> and <code>/* */</code>)
are supported in JSONC files. The <code>background</code>/<code>foreground</code>
keys are optional for third-party files written against the old 16-slot
format: absent keys fall back to <code>palette[0]</code> /
<code>palette[7]</code>. The palette entries are not arbitrary colors: every
slot has a fixed <strong>role</strong> (see the
<a href="#palette-reference">Palette Reference</a>), so themes stay
interchangeable and every renderer — widget packs and kernel chrome alike —
picks colors by role, never by taste.</p>

<pre><code>{
    // my-custom-theme -- Dark background, warm accents
    "name": "my-custom-theme",
    "background": "#1a1b1c", // screen/frame background (role bg)
    "foreground": "#abb2bf", // primary text (role fg)
    "palette": [
        "#1a1b1c", //  0: legacy background alias (ROLE_BG)
        "#e06c75", //  1: alert red (high fills, avg cpu line)
        "#98c379", //  2: good green (normal fills, RAM line)
        "#e5c07b", //  3: warn yellow (gradient mid stop)
        "#d19a66", //  4: read / RX (network RX, disk reads)
        "#c678dd", //  5: write / TX / GPU (network TX, disk writes)
        "#56b6c2", //  6: accent (titles, headers, selection)
        "#abb2bf", //  7: legacy foreground alias (ROLE_FG)
        "#3e4451", //  8: dim (zebra rows, separators, dividers)
        "#e06c75", //  9..15: bright series ramp (multi-series charts)
        "#98c379",
        "#e5c07b",
        "#d19a66",
        "#c678dd",
        "#56b6c2",
        "#abb2bf"
    ]
}</code></pre>

<h3 id="contrast-normalization">Contrast normalization (UX8.2)</h3>

<p>Every theme is normalized once, right after parsing, against its explicit
<code>background</code>. The engine measures the WCAG contrast ratio of each
role and auto-lifts the colors that fail their floor, deterministically and
hue-preserving (colors move toward white on dark backgrounds, toward black
on light ones, in small steps until the floor clears):</p>

<table>
  <thead>
    <tr><th>Role</th><th>Floor</th><th>Source</th></tr>
  </thead>
  <tbody>
    <tr><td>foreground text</td><td>4.5:1</td><td>explicit <code>foreground</code> (legacy files: slot 7)</td></tr>
    <tr><td>accent</td><td>3.0:1</td><td>slot 6</td></tr>
    <tr><td>dim</td><td>3.0:1</td><td>slot 8</td></tr>
    <tr><td>zebra-row text</td><td>3.0:1</td><td>foreground painted over the dim stripe (slots share 8)</td></tr>
    <tr><td>series ramp / accents</td><td>2.0:1</td><td>slots 1–5, 7, 9–15 (colored marks on the background)</td></tr>
  </tbody>
</table>

<p>The dim floor and the zebra-row-text floor share slot 8, so they cannot
both hold on palettes whose foreground sits close to the background:
zebra-row text always wins, and dim keeps the highest value that still
clears it (for the shipped <code>helsinki</code>/<code>oslo</code> palettes
no lift is possible at all and dim keeps its canonical value). The lifted
values replace the in-memory palette entries, so renderers keep reading
<code>theme_palette()</code> and the role accessors unchanged — the
<strong>shipped/user files are never rewritten</strong>: normalization
happens at load only.</p>

<h3 id="palette-reference">Palette Reference</h3>

<p>This table is the single truthful role reference: the kernel theme
accessors (<code>bg()</code>/<code>fg()</code>/<code>accent()</code>/<code>dim()</code> in
<code>src/theme/model.rs</code>) and the widget packs' <code>ROLE_*</code> constants
(widgets repo, <code>src/util.rs</code>) map to exactly these slots, and the usage
column reflects what the code really paints today. If a renderer needs a
color, it takes it from here — no undocumented palette index. Text is drawn
with the explicit <code>foreground</code>/<code>background</code> pair
(<code>theme_fg()</code>/<code>theme_bg()</code> on the widget contract);
the palette slots feed the colored marks.</p>

<table>
  <thead>
    <tr>
      <th>Index</th>
      <th>Role</th>
      <th>Actual usage (kernel + widget packs)</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>0</code></td>
      <td>legacy background alias (<code>ROLE_BG</code>)</td>
      <td>Terminal/block background of the packs' frames; the screen itself paints the explicit <code>background</code> key (<code>Theme::bg()</code>)</td>
    </tr>
    <tr>
      <td><code>1</code></td>
      <td>alert (<code>ROLE_ALERT</code>)</td>
      <td>Red/high fills: CPU/mem/swap gauges past their alert threshold, the CPU average chart line, the minimal-view CPU gauge</td>
    </tr>
    <tr>
      <td><code>2</code></td>
      <td>good (<code>ROLE_GOOD</code>)</td>
      <td>Green/normal fills: low gradient stop, RAM history line, battery fill, minimal-view memory gauge</td>
    </tr>
    <tr>
      <td><code>3</code></td>
      <td>warn (<code>ROLE_WARN</code>)</td>
      <td>Yellow fills: gradient mid stop for CPU/mem/storage/swap at or above 50% but below the alert threshold</td>
    </tr>
    <tr>
      <td><code>4</code></td>
      <td>read/RX (<code>ROLE_RX</code>)</td>
      <td>Download/read metrics: network RX totals/lines, disk_io read gauges</td>
    </tr>
    <tr>
      <td><code>5</code></td>
      <td>write/TX (<code>ROLE_TX</code>)</td>
      <td>Upload/write metrics: network TX totals/lines, disk_io write gauges, GPU fill</td>
    </tr>
    <tr>
      <td><code>6</code></td>
      <td>accent (<code>ROLE_ACCENT</code>)</td>
      <td>Accents: process table header/selection, help key spans, overlay titles/borders; <code>Theme::accent()</code></td>
    </tr>
    <tr>
      <td><code>7</code></td>
      <td>legacy foreground alias (<code>ROLE_FG</code>)</td>
      <td>Near-white anchor of the base hue family; can legitimately equal the background (Paris slot 7 == its background — renderers draw text with the explicit <code>foreground</code>, <code>Theme::fg()</code>)</td>
    </tr>
    <tr>
      <td><code>8</code></td>
      <td>dim (<code>ROLE_DIM</code>)</td>
      <td>Dim/secondary: zebra row backgrounds, column separators, chart dividers, muted notes; <code>Theme::dim()</code></td>
    </tr>
    <tr>
      <td><code>9</code>–<code>15</code></td>
      <td>bright series ramp (<code>ROLE_SERIES_START</code>..<code>ROLE_SERIES_END</code>)</td>
      <td>Seven bright variants for multi-series charts (per-core history lines, cycle by series index)</td>
    </tr>
  </tbody>
</table>

<p>The shipped themes repeat the base hue family in slots 9–15 (bright
variants of slots 1–7 in the same order), which is what makes the series
ramp look coherent. Keep that convention when writing a theme: slots 9–15
should stay distinguishable from each other and from slots 1–8.</p>

<h3 id="starter-themes">Starter Themes</h3>

<p>xtop ships <strong>12 themes</strong>. The <code>x</code> palette (almost-black
background, purple-pink accents) is compiled into the binary as the startup
fallback; all 12 definitions — including <code>x</code> and <code>miami</code> —
are embedded in the binary as seeding templates. The first run writes them
into the themes directory above, so every shipped theme is available without
copying anything.</p>

<p>If you want to restore them later, copy from the repository:</p>

<pre><code>cp -r assets/themes/* ~/.config/xtop/themes/   # Linux
# macOS: ~/Library/Application Support/xtop/themes/
</code></pre>

<p><strong>Available themes:</strong> <code>x</code>, <code>berlin</code>, <code>bogota</code>,
<code>helsinki</code>, <code>lahabana</code>, <code>london</code>, <code>madrid</code>,
<code>miami</code>, <code>oslo</code>, <code>paris</code>, <code>praha</code>,
<code>tokio</code>.</p>

<p>All theme palettes are documented in <a href="colors.md"><code>colors.md</code></a>.</p>

<h3 id="switching-themes-from-the-cli">Switching Themes from the CLI</h3>

<p><code>xtop --ct &lt;name&gt;</code> changes the active theme and persists it
to <code>config.json</code>, so the next start uses it. A running instance
follows the change live within one tick because the TUI run loop polls the
persisted theme — no restart and no IPC needed. This is the hook for
external themers: e.g. a Hyprland theme switcher script can run
<code>xtop --ct tokio</code> alongside its own theme changes. Unknown names
fail with the list of available themes.</p>

<h3 id="loading-order">Loading Order</h3>

<ol>
  <li>The compiled-in <code>x</code> palette (startup fallback, index 0).</li>
  <li>Themes from the themes directory (seeded on first run) load on top;
      a file reusing the name <code>x</code> overrides the compiled palette in
      place.</li>
  <li>If a custom theme has the same name as a shipped one, it <strong>replaces</strong>
      it; new names are appended after the shipped set.</li>
</ol>

<h3 id="theme-tips">Tips</h3>

<ul>
  <li>Try the grayscale themes (<code>london</code>, <code>berlin</code>) as a base and add your own accent colors.</li>
  <li>Prefer writing the explicit <code>background</code>/<code>foreground</code> keys: the fallback (slot 0/slot 7) is only for legacy files, and a slot-7 foreground can equal the background (the shipped Paris palette keeps that quirk on purpose — the explicit pair is what the roles are anchored on).</li>
  <li>Do not fight the contrast normalizer: roles below their floor are lifted at load. Write the palette you want; the engine guarantees the floors in-memory while the file stays canonical.</li>
  <li>Slots 9–15 are the bright series ramp: make each entry a brighter sibling of slots 1–7 in order so multi-series charts stay distinct.</li>
  <li>Never repurpose a slot: renderers pick colors by the role table above, so a theme that reorders roles breaks every widget that reads them.</li>
</ul>

<hr>

<h2 id="layouts">Layouts</h2>

<h3 id="layout-location">Location</h3>

<p>Place layout files in the platform layouts directory (config dir +
<code>layouts</code>):</p>

<pre><code>~/.config/xtop/layouts/*.jsonc          (Linux)
~/Library/Application Support/xtop/layouts/*.jsonc   (macOS)
%APPDATA%\xtop\layouts\*.jsonc          (Windows)
</code></pre>

<p>Both <code>.jsonc</code> and <code>.json</code> extensions are accepted.</p>

<h3 id="layout-format">Format</h3>

<p>A layout is a recursive tree of <strong>splits</strong> and <strong>widgets</strong>:</p>

<pre><code>LayoutDef
 ├── name: string
 └── root: Area
      ├── direction: "horizontal" | "vertical"
      ├── size: constraint (optional, defaults to "*")
      └── areas: [Area, ...]
           ├── Area with "widget" → leaf node (renders a widget)
           └── Area with "direction" → nested split
</code></pre>

<h3 id="size-constraints">Size Constraints</h3>

<table>
  <thead>
    <tr>
      <th>Syntax</th>
      <th>Meaning</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>"*"</code> or omitted</td>
      <td>Fill remaining space</td>
    </tr>
    <tr>
      <td><code>3</code> (number)</td>
      <td>Fixed <em>n</em> rows/columns</td>
    </tr>
    <tr>
      <td><code>"45%"</code></td>
      <td>Percentage of parent</td>
    </tr>
  </tbody>
</table>

<h3 id="available-widgets">Available Widgets</h3>

<table>
  <thead>
    <tr>
      <th>Widget</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>header</code></td>
      <td>System info bar (uptime, load, keys)</td>
    </tr>
    <tr>
      <td><code>cpu</code></td>
      <td>Per-core CPU usage gauges</td>
    </tr>
    <tr>
      <td><code>memory</code></td>
      <td>RAM + Swap gauges + RAM history chart</td>
    </tr>
    <tr>
      <td><code>storage</code></td>
      <td>Disk usage gauges per mount point</td>
    </tr>
    <tr>
      <td><code>network</code></td>
      <td>Network RX/TX totals and speeds</td>
    </tr>
    <tr>
      <td><code>processes</code></td>
      <td>Process table with search filter</td>
    </tr>
    <tr>
      <td><code>disk_io</code></td>
      <td>Disk read/write speeds</td>
    </tr>
    <tr>
      <td><code>battery</code></td>
      <td>Battery charge gauges</td>
    </tr>
    <tr>
      <td><code>gpu</code></td>
      <td>GPU usage gauges</td>
    </tr>
  </tbody>
</table>

<h3 id="layout-examples">Examples</h3>

<h4>Simple custom layout</h4>

<p>A minimal three-row layout: header, CPU, and processes.</p>

<pre><code>{
    // "monitor" — CPU top-half, processes bottom-half
    "name": "monitor",
    "root": {
        "direction": "vertical",
        "areas": [
            { "widget": "header", "size": 3 },
            { "widget": "cpu", "size": "55%" },
            { "widget": "processes", "size": "*" }
        ]
    }
}</code></pre>

<h4>Complex nested layout</h4>

<p>A full dashboard with a horizontal split in the middle section:</p>

<pre><code>{
    "name": "my-dashboard",
    "root": {
        "direction": "vertical",
        "areas": [
            { "widget": "header", "size": 3 },
            {
                "direction": "horizontal",
                "size": "50%",
                "areas": [
                    { "widget": "cpu", "size": "60%" },
                    {
                        "direction": "vertical",
                        "size": "40%",
                        "areas": [
                            { "widget": "network", "size": "50%" },
                            { "widget": "disk_io", "size": "50%" }
                        ]
                    }
                ]
            },
            { "widget": "processes", "size": "*" }
        ]
    }
}</code></pre>

<h3 id="starter-layouts">Starter Layouts</h3>

<p>The 10 built-in layouts ship in the <code>xtop-layout</code> crate
(<code>github.com/xtop-cli/layouts</code>, folder <code>layouts/default/</code>) and are embedded in the
binary. On startup their JSONC sources are copied to the platform layouts
directory (see <a href="#layout-location">Location</a>) as editable templates. Community layouts
live in <code>layouts/custom/</code> of the same repo;
install one with <code>xtop layout install &lt;name&gt;</code> (or copy the file into
the layouts directory for your platform). Validate a local file with <code>xtop layout check &lt;file&gt;</code>.</p>

<p>A layout file whose <code>name</code> matches a built-in layout <strong>overrides it</strong> (e.g.
edit <code>dashboard.jsonc</code> to customize the Dashboard). Files with new names show up as
extra layouts.</p>

<p><strong>Mode layouts:</strong> <code>dashboard</code>, <code>vertical</code>, <code>horizontal</code>, <code>cpu_focus</code>, <code>memory_focus</code>, <code>network_focus</code>, <code>process_focus</code> — these seven map to the layout modes.</p>

<p><strong>Preset extras:</strong> <code>detail_dashboard</code>, <code>detail_network</code>, <code>detail_processes</code> — detail-focused layouts appended after the modes (not modes themselves; they are selected by name). They exercise per-widget display options (CPU basis on <code>processes</code>, <code>cores</code>/<code>show_freq</code> on <code>cpu</code>, <code>ifaces</code> on <code>network</code>); see the per-widget options section below.</p>

<h3 id="cycling-order">Cycling Order</h3>

<ol>
  <li>Mode layouts (Dashboard → Vertical → Horizontal → CPU Focus → Memory Focus → Network Focus → Process Focus)</li>
  <li>Preset extras (<code>Detail Dashboard</code> → <code>Detail Network</code> → <code>Detail Processes</code>)</li>
  <li>Any custom layout from the platform layouts directory with a new name (filesystem order)</li>
  <li>Custom files that reuse a built-in <code>name</code> override that built-in in place (no duplicates)</li>
  <li>Wraps back to Dashboard</li>
</ol>

<p>Press <kbd>l</kbd> to cycle forward through all available layouts.</p>

<h3 id="layout-notes">Notes</h3>

<ul>
  <li>If a widget name in your layout doesn't match any available widget, that area is skipped and xtop prints a one-time warning to stderr (<code>xtop: layout '&lt;layout&gt;' references unknown widget '&lt;name&gt;'</code>).</li>
  <li>Nested splits can be arbitrarily deep, but very deep nesting may overflow small terminals.</li>
  <li>The terminal must be at least 40×8 for any layout to render; smaller terminals show a warning.</li>
  <li>Very small terminals (under 60×14) fall back to a minimal hardcoded layout (CPU + Memory gauges + process list).</li>
</ul>

<h3 id="glyph-style">Widget glyph style</h3>

<p>Charts (CPU/Memory/Network) and widget borders are drawn with glyph styles
you can change in <code>config.json</code> under the <code>style</code> key (see
<a href="configuration.md">configuration.md</a>):</p>

<pre><code>{
  "theme": "x",
  "style": {
    "charset": "block",
    "borders": "ascii",
    "widgets": {
      "cpu": { "charset": "bar" },
      "network": { "borders": "double" }
    }
  }
}</code></pre>

<ul>
  <li><code>charset</code>: <code>braille</code> (default), <code>dot</code>, <code>block</code>, <code>half_block</code>, <code>bar</code>.</li>
  <li><code>borders</code>: <code>native</code> (default; the classic single-line box-drawing frame), <code>rounded</code>, <code>double</code>, <code>plain</code> and <code>ascii</code> (both plain and ascii draw a pure ASCII <code>+-|</code> frame).</li>
  <li><code>widgets</code>: per-widget overrides. Keys are the widget names layouts
      use: <code>header</code>, <code>cpu</code>, <code>memory</code>, <code>storage</code>,
      <code>network</code>, <code>processes</code>, <code>disk_io</code>, <code>battery</code>, <code>gpu</code>.
      Each entry accepts <code>charset</code>, <code>borders</code> and an optional
      <code>pack</code> (widget pack to render that name with, e.g. <code>"blocks"</code>).
      A global <code>style.pack</code> sets the pack for every widget without a
      per-widget override.</li>
</ul>

<p>Glyph styles only change the look: the data behind each widget is drawn by
the widget packs (see <a href="plugin.md">plugin.md</a> for how a plugin adds
completely new renderers, which take precedence over packs).</p>

<h3 id="per-widget-display-options">Per-widget display options</h3>

<p>Beyond glyph style, every widget <em>instance</em> in a layout file can carry
an <code>options</code> JSON object that refines how that instance draws its
data. The layout format accepts it on widget nodes as an opaque passthrough
(see the layouts repo, <code>docs/layout-schema.md</code>, section "Widget
<code>options</code>"):</p>

<pre><code>{
  "name": "My Layout",
  "root": {
    "direction": "vertical",
    "areas": [
      { "widget": "header", "size": 3 },
      { "widget": "cpu", "size": "60%", "options": { "cores": "all" } },
      { "widget": "processes", "size": "*" }
    ]
  }
}</code></pre>

<ul>
  <li>The kernel forwards each node's <code>options</code> object to the
      widget's renderer while that instance is drawn (via
      <code>WidgetState::widget_options</code> in the widget-api contract).
      Multiple instances of the same widget in one layout can carry
      different options.</li>
  <li>No <code>options</code> key (or <code>null</code>) means the widget
      renders exactly as before this feature — the defaults preserve the
      current behavior byte-for-byte. Only documented keys refine a widget;
      unknown keys are ignored.</li>
  <li>Recognized keys are documented per widget in the widgets repo
      (<code>docs/widgets.md</code>) as the UX milestones land them. The
      shipped Detail presets showcase the first wave: the
      <code>processes</code>
      CPU basis (<code>cpu</code>: <code>"total"</code>/<code>"both"</code>),
      the <code>cpu</code> core/frequency keys
      (<code>cores</code>, <code>show_freq</code>) and the
      <code>network</code> interface list (<code>ifaces</code>). Until a
      widget documents a key, that key is inert (DR-UX2 defaults).</li>
  <li>Fullscreen and minimal views look the options up by widget name in the
      current layout (first matching node); when the layout has no such node
      they use the defaults.</li>
  <li>Plugin widget renderers see the plugin <code>HostState</code>, not
      <code>WidgetState</code>: layout <code>options</code> are not forwarded
      to plugins.</li>
</ul>

<hr>

<h2 id="widget-packs">Widget Packs</h2>

<p>Widget packs are the installable unit of widget code: each pack is a
separate crate (<code>xtop-widget-&lt;name&gt;</code>) that registers
renderers by widget name against the <code>xtop-widget-api</code> contract.
The kernel ships two packs out of the box — the base pack
(<code>default</code>, always compiled in) and the <code>blocks</code> pack
(gated behind the <code>widget-blocks</code> Cargo feature) — and lists them
in a single compile-time catalog
(<code>src/ui/layout/pack_table.rs</code>, one <code>(feature, label)</code>
row per pack), which the render engine and <code>xtop widget list</code>
share.</p>

<p>Widget-pack management mirrors the plugin workflow:</p>

<table>
  <thead>
    <tr><th>Command</th><th>Description</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>xtop widget list</code></td>
      <td>List the widget packs wired into the kernel (pack-table rows whose Cargo feature is declared in the root <code>Cargo.toml</code>)</td>
    </tr>
    <tr>
      <td><code>xtop widget scaffold &lt;name&gt;</code></td>
      <td>Create a compiling single-widget pack template in <code>widgets-dev/xtop-widget-&lt;name&gt;/</code> (git-ignored)</td>
    </tr>
    <tr>
      <td><code>xtop widget install &lt;name&gt;</code></td>
      <td>Install a pack by name from <code>github.com/xtop-cli/widgets</code></td>
    </tr>
    <tr>
      <td><code>xtop widget install &lt;url|path&gt;</code></td>
      <td>Install a pack from any git URL or a local crate directory</td>
    </tr>
  </tbody>
</table>

<p><code>xtop widget install</code> is a self-modifying-source workflow (the
same spirit as <code>xtop plugin install</code>): it adds an optional
dependency + a <code>widget-&lt;name&gt;</code> feature flag to the root
<code>Cargo.toml</code>, appends one <code>(feature, label)</code> row and
its registry-linking arm to the pack catalog in
<code>src/ui/layout/pack_table.rs</code>, and runs <code>cargo check</code>.
The pack is <strong>not enabled by default</strong>: add
<code>widget-&lt;name&gt;</code> to the <code>[features]</code> default list
(or build with <code>--features widget-&lt;name&gt;</code>) and rebuild.
Once enabled, select the pack per widget with <code>style.pack</code> (all
widgets) or <code>style.widgets.&lt;name&gt;.pack</code> (one widget), then
place its widget names in a layout file. Authoring guidance (the pack
contract, how packs register renderers, the renderers' options) lives in the
widgets repo docs (<code>docs/authoring.md</code>, <code>docs/widgets.md</code>).</p>

<hr>

<h2 id="runtime-widgets">Runtime Widgets (WASM / External Processes)</h2>

<p>Besides compiled-in packs, xtop can host <strong>runtime widgets</strong>:
code that is not compiled into the kernel and is loaded from the user config
directory at startup. Two optional hosts exist, both off by default:</p>

<table>
  <thead>
    <tr><th>Feature</th><th>Source</th><th>Directory</th><th>Isolation</th></tr>
  </thead>
  <tbody>
    <tr>
      <td><code>plugin-wasm</code></td>
      <td><code>.wasm</code> modules loaded with wasmi</td>
      <td><code>wasm/</code> (override <code>XTOP_WASM_DIR</code>)</td>
      <td>In-process sandbox: fuel budget per call, 64&nbsp;MiB memory cap, no host imports except <code>host.log</code></td>
    </tr>
    <tr>
      <td><code>plugin-external</code></td>
      <td>One helper process per widget (Lua, Python, Node, any language)</td>
      <td><code>external/</code> (override <code>XTOP_EXTERNAL_DIR</code>)</td>
      <td>The process boundary itself (user permissions); every response is bounded by <code>timeout_ms</code></td>
    </tr>
  </tbody>
</table>

<p>Build with the hosts enabled:</p>

<pre><code>cargo build --release --features plugin-wasm,plugin-external</code></pre>

<p>Runtime widgets register through the plugin widget path, so they keep
precedence over every pack and can replace any widget name. Layouts reference
them by the <code>name</code> in the guest manifest (for example
<code>wasm-clock</code>). A guest is called once per tick and its draw list
is cached and replayed at render time, so a slow guest can never stall a
frame; WASM modules hot-reload when their file changes and helper processes
keep their last good frame when they time out or fail.</p>

<p>Working examples ship in the plugins repo: Rust WASM guests under
<code>examples/wasm/</code> (clock, CPU gauge + sparkline, process table) and
Lua/Python/Node helper widgets under <code>examples/external/</code>. The
full contracts — guest ABI, draw-list operations, state snapshot fields and
the external descriptor format — live in <code>docs/wasm-widgets.md</code>
and <code>docs/external-widgets.md</code> of that repo. The workspace ships a
self-contained demo that builds and installs everything into
<code>temp/xtop-demo/</code> and launches xtop against it:
<code>./temp/xtop-demo/run-demo.sh</code>.</p>

<hr>

<p align="center">
  <a href="../README.md">← Back to README</a>
</p>
