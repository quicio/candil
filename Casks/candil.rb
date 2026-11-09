cask "candil" do
  version :latest
  sha256 :no_check

  url "https://github.com/quicio/candil/releases/latest/download/candil-macos-arm64-app.tar.gz"
  name "Candil"
  desc "Lector de EPUB minimalista"
  homepage "https://github.com/quicio/candil"

  depends_on arch: :arm64
  depends_on :macos

  app "Candil.app"

  caveats <<~EOS
    Candil se distribuye sin notarización de Apple. Si macOS bloquea la primera
    apertura, instalá con:

      brew install --cask --no-quarantine quicio/tap/candil

    o usá clic derecho → Abrir sobre Candil.app.
  EOS
end
