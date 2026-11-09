cask "candil" do
  version :latest
  sha256 :no_check

  url "https://github.com/quicio/candil/releases/latest/download/candil-macos-arm64-app.zip"
  name "Candil"
  desc "Lector de EPUB minimalista"
  homepage "https://github.com/quicio/candil"

  depends_on macos: ">= :big_sur"
  depends_on arch: :arm64

  app "Candil.app"

  # Candil no está notarizado por Apple: el zip se firma ad-hoc. Homebrew
  # conserva la cuarentena de la descarga, así que la quitamos para que la
  # primera apertura no quede bloqueada por Gatekeeper.
  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-dr", "com.apple.quarantine", "#{appdir}/Candil.app"],
                   must_succeed: false
  end

  caveats <<~EOS
    Candil se distribuye sin notarización de Apple. Este cask intenta quitar la
    cuarentena; si macOS igual bloquea la primera apertura, usá clic derecho →
    Abrir sobre Candil.app.
  EOS
end
