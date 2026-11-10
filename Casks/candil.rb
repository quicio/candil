cask "candil" do
  version :latest
  sha256 :no_check

  on_arm do
    url "https://github.com/quicio/candil/releases/latest/download/candil-macos-arm64-app.tar.gz"
  end
  on_intel do
    url "https://github.com/quicio/candil/releases/latest/download/candil-macos-x86_64-app.tar.gz"
  end

  name "Candil"
  desc "Lector de EPUB minimalista"
  homepage "https://github.com/quicio/candil"

  depends_on :macos

  app "Candil.app"

  # Candil no está notarizado por Apple: el zip se firma ad-hoc. Homebrew
  # conserva la cuarentena de la descarga, así que la quitamos para que la
  # primera apertura no quede bloqueada por Gatekeeper.
  postflight_steps do
    run "/usr/bin/xattr",
        args:           ["-dr", "com.apple.quarantine", "/Applications/Candil.app"],
        writable_paths: ["/Applications/Candil.app"],
        must_succeed:   false
  end

  caveats <<~EOS
    Candil se distribuye sin notarización de Apple. Este cask intenta quitar la
    cuarentena; si macOS igual bloquea la primera apertura, usá clic derecho →
    Abrir sobre Candil.app.
  EOS
end
