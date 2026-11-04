<h1>Installation Guide</h1>

<hr>

<h2 id="table-of-contents">Table of Contents</h2>

<ul>
  <li><a href="#prebuilt">Prebuilt Binaries (recommended)</a></li>
  <li><a href="#linux-macos-prebuilt">Linux and macOS</a></li>
  <li><a href="#windows-prebuilt">Windows</a></li>
  <li><a href="#prebuilt-options">Installer Options and Verification</a></li>
  <li><a href="#release-assets">Release Assets</a></li>
  <li><a href="#build-from-source">Install from Source</a></li>
  <li><a href="#uninstall">Uninstall</a></li>
  <li><a href="#supported-distributions">Supported Distributions</a></li>
</ul>

<hr>

<h2 id="prebuilt">Prebuilt Binaries (recommended)</h2>

<p>The release workflow publishes precompiled binaries. Installing them needs
no Rust toolchain and no compilation. The installers download the archive for
the current platform, verify the release <code>SHA256SUMS</code> and, when
<code>cosign</code> is available, the keyless signature bundle.</p>

<p>Supported prebuilt targets:</p>

<ul>
  <li>Linux x86_64, glibc (<code>x86_64-unknown-linux-gnu</code>)</li>
  <li>Linux x86_64, musl (<code>x86_64-unknown-linux-musl</code>), detected automatically on Alpine and other musl systems</li>
  <li>Linux aarch64, glibc (<code>aarch64-unknown-linux-gnu</code>)</li>
  <li>macOS x86_64 (<code>x86_64-apple-darwin</code>)</li>
  <li>macOS aarch64 / Apple Silicon (<code>aarch64-apple-darwin</code>)</li>
  <li>Windows x86_64 (<code>x86_64-pc-windows-msvc</code>)</li>
</ul>

<hr>

<h2 id="linux-macos-prebuilt">Linux and macOS</h2>

<pre><code>curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install-prebuilt.sh | bash</code></pre>

<p>Or, with wget:</p>

<pre><code>wget -qO- https://raw.githubusercontent.com/xtop-cli/xtop/main/install-prebuilt.sh | bash</code></pre>

<p>Pin a version, choose a prefix or install system-wide:</p>

<pre><code># Specific version
curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install-prebuilt.sh | bash -s -- --version 0.1.0

# User prefix (default: ~/.local; binary in ~/.local/bin)
bash install-prebuilt.sh --prefix "$HOME/.local" --yes

# System-wide (requires write access to /usr/local/bin)
bash install-prebuilt.sh --prefix /usr/local --yes

# Custom binary directory
bash install-prebuilt.sh --bin-dir "$HOME/bin"
</code></pre>

<p>If the binary directory is not on <code>PATH</code>, the installer prints
the line to add to the shell profile.</p>

<hr>

<h2 id="windows-prebuilt">Windows</h2>

<p>Run in PowerShell:</p>

<pre><code>irm https://raw.githubusercontent.com/xtop-cli/xtop/main/install-prebuilt.ps1 | iex</code></pre>

<p>The default install directory is
<code>%LOCALAPPDATA%\Programs\xtop\bin</code>. The installer adds it to the
user <code>PATH</code> unless <code>-NoPath</code> is given.</p>

<pre><code># Specific version
.\install-prebuilt.ps1 -Version 0.1.0

# Custom binary directory
.\install-prebuilt.ps1 -BinDir C:\tools\bin

# Skip the PATH update
.\install-prebuilt.ps1 -NoPath
</code></pre>

<hr>

<h2 id="prebuilt-options">Installer Options and Verification</h2>

<p>Unix installer options:</p>

<table>
  <tr><th>Option</th><th>Meaning</th></tr>
  <tr><td><code>--version &lt;ver&gt;</code></td><td>Install a specific release instead of the latest</td></tr>
  <tr><td><code>-p, --prefix &lt;dir&gt;</code></td><td>Install prefix (default: <code>$HOME/.local</code>)</td></tr>
  <tr><td><code>-b, --bin-dir &lt;dir&gt;</code></td><td>Binary directory (default: <code>$PREFIX/bin</code>)</td></tr>
  <tr><td><code>-y, --yes</code></td><td>Non-interactive</td></tr>
  <tr><td><code>-q, --quiet</code></td><td>Minimal output</td></tr>
  <tr><td><code>--require-signature</code></td><td>Abort unless cosign verifies the signature bundle</td></tr>
  <tr><td><code>--no-checksum</code></td><td>Disable SHA-256 and signature verification (dangerous)</td></tr>
</table>

<p>Windows installer parameters: <code>-Version</code>, <code>-BinDir</code>,
<code>-NoChecksum</code> and <code>-NoPath</code>.</p>

<p>Every release publishes <code>SHA256SUMS</code>. The installers refuse to
install an archive that is not listed or whose hash does not match. When
<code>cosign</code> is installed, the keyless signature is verified as well.
Manual verification:</p>

<pre><code>cosign verify-blob \
  --bundle SHA256SUMS.bundle \
  --certificate-identity "https://github.com/xtop-cli/xtop/.github/workflows/release.yml@refs/tags/v0.1.0" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  SHA256SUMS
sha256sum -c SHA256SUMS
</code></pre>

<hr>

<h2 id="release-assets">Release Assets</h2>

<p>Each release publishes one archive per target plus the checksum files:</p>

<pre><code>xtop-&lt;version&gt;-&lt;target&gt;.tar.gz   (Unix)
xtop-&lt;version&gt;-&lt;target&gt;.zip      (Windows)
SHA256SUMS
SHA256SUMS.bundle
</code></pre>

<p>Archives contain the <code>xtop</code> binary (or <code>xtop.exe</code>)
plus <code>LICENSE</code>, <code>README.md</code> and
<code>CHANGELOG.md</code>.</p>

<hr>

<h2 id="build-from-source">Install from Source</h2>

<h3>Linux</h3>

<p>The source installer detects the distribution and package manager, installs
the build prerequisites (git and, when missing, Rust through rustup), clones
the repository, builds in release mode and installs the binary to
<code>/usr/local/bin</code>.</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install.sh | bash</code></pre>

<pre><code># Check dependencies without installing
./install.sh --check-deps

# Install only dependencies (Rust, build tools)
./install.sh --install-deps

# Install with the runtime widget hosts enabled
./install.sh --with-runtime-widgets

# Show help
./install.sh --help
</code></pre>

<h3>Windows</h3>

<p>Requires <a href="https://rustup.rs/">Rust (Cargo)</a>:</p>

<pre><code>irm https://raw.githubusercontent.com/xtop-cli/xtop/main/install.ps1 | iex
.\install.ps1 -RuntimeWidgets
</code></pre>

<h3>macOS and other platforms</h3>

<pre><code>cargo install --git https://github.com/xtop-cli/xtop --all-features</code></pre>

<p>The binary lands in <code>~/.cargo/bin</code>; make sure it is on your
PATH. <code>--all-features</code> enables the Samurai plugin, the MCP
extension, the blocks widget pack, the effects module and the two runtime
widget hosts (sandboxed WASM and external processes).</p>

<h3>From a clone</h3>

<ol>
  <li>
    <p>Clone the repository:</p>
    <pre><code>git clone https://github.com/xtop-cli/xtop.git
cd xtop</code></pre>
  </li>
  <li>
    <p>Build and run with release optimizations:</p>
    <pre><code>cargo run --release</code></pre>
  </li>
</ol>

<hr>

<h2 id="uninstall">Uninstall</h2>

<p>For a prebuilt install, remove the binary (and the directory if it is
empty):</p>

<pre><code>rm -f "$HOME/.local/bin/xtop"</code></pre>

<pre><code>Remove-Item "$env:LOCALAPPDATA\Programs\xtop" -Recurse -Force</code></pre>

<p>The source uninstallers are also available:</p>

<h3>Linux</h3>

<pre><code>curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install.sh | bash -s -- --uninstall</code></pre>

<h3>Windows</h3>

<pre><code>irm https://raw.githubusercontent.com/xtop-cli/xtop/main/uninstall.ps1 | iex</code></pre>

<p>Uninstallers remove the binary only; user configuration under the config
directory is preserved.</p>

<hr>

<h2 id="supported-distributions">Supported Distributions</h2>

<p>The source installer supports:</p>

<ul>
  <li>Arch Linux and derivatives</li>
  <li>Debian / Ubuntu and derivatives</li>
  <li>Fedora / RHEL and derivatives</li>
  <li>openSUSE and derivatives</li>
  <li>Alpine Linux and derivatives</li>
</ul>

<hr>

<p align="center">
  <a href="../README.md">Back to README</a>
</p>
