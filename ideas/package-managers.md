# Package Manager Distribution

This document outlines the effort required to publish cimishi to various package managers.

## Effort Summary

| Platform | Effort | Maintenance |
|----------|--------|-------------|
| **Homebrew** | Low | Low |
| **Chocolatey** | Medium | Medium |
| **Winget** | Low-Medium | Low |
| **APT (deb)** | Medium-High | Medium |

---

## Homebrew (macOS/Linux)

**Effort: ~1-2 hours initial setup**

Create a tap repository (e.g., `topswagcode/homebrew-tap`) with a formula:

```ruby
# filepath: Formula/cimishi.rb
class Cimishi < Formula
  desc "CIM/RDF CLI tool for SPARQL queries"
  homepage "https://github.com/topswagcode/power-test"
  version "0.0.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/topswagcode/power-test/releases/download/v#{version}/cimishi-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_HERE"
    end
  end

  on_linux do
    url "https://github.com/topswagcode/power-test/releases/download/v#{version}/cimishi-x86_64-unknown-linux-musl.tar.gz"
    sha256 "SHA256_HERE"
  end

  def install
    bin.install "cimishi"
  end
end
```

Add a GitHub Action step to auto-update the formula on release.

---

## Winget (Windows)

**Effort: ~2-3 hours initial setup**

1. Fork [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs)
2. Create manifest files:

```yaml
# manifests/t/topswagcode/cimishi/0.0.1/topswagcode.cimishi.yaml
PackageIdentifier: topswagcode.cimishi
PackageVersion: 0.0.1
PackageLocale: en-US
Publisher: topswagcode
PackageName: cimishi
License: MIT
ShortDescription: CIM/RDF CLI tool
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/.../cimishi-x86_64-pc-windows-msvc.zip
    InstallerSha256: SHA256_HERE
    InstallerType: zip
ManifestType: singleton
ManifestVersion: 1.0.0
```

3. Submit PR (Microsoft reviews within ~1 week)

---

## Chocolatey (Windows)

**Effort: ~3-4 hours + account setup**

Requires creating a `.nuspec` and `chocolateyInstall.ps1`:

```powershell
# tools/chocolateyInstall.ps1
$url = 'https://github.com/.../cimishi-x86_64-pc-windows-msvc.zip'
Install-ChocolateyZipPackage 'cimishi' $url "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
```

Needs Chocolatey account and package moderation (~1-2 days review).

---

## Debian/Ubuntu (.deb)

**Effort: ~4-6 hours**

Requires building proper `.deb` packages with:
- Control files (package metadata)
- Post-install scripts
- Proper directory structure

Add to your release workflow:

```yaml
- name: Build deb package
  if: matrix.target == 'x86_64-unknown-linux-musl'
  run: |
    mkdir -p pkg/usr/bin pkg/DEBIAN
    cp target/${{ matrix.target }}/release/cimishi pkg/usr/bin/
    cat > pkg/DEBIAN/control << EOF
    Package: cimishi
    Version: ${{ github.ref_name }}
    Architecture: amd64
    Maintainer: you@example.com
    Description: CIM/RDF CLI tool
    EOF
    dpkg-deb --build pkg cimishi_${{ github.ref_name }}_amd64.deb
```

For official Ubuntu/Debian repos, you'd need a PPA (Personal Package Archive) which adds significant complexity.

---

## Recommendation

Start with **Homebrew** (easiest, covers macOS + Linux) and **Winget** (straightforward PR process). These give you the best coverage with minimal maintenance.
