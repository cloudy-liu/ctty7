"""Render tty7's raster icons from the SVG sources in assets/.

    pip install resvg-py pillow icnsutil
    python scripts/brand/render.py

The SVGs are the source of truth: app-icon.svg is the tile on the macOS icon
grid, logo.svg is the same drawing cropped to the tile, and tray.svg is read
by the app at runtime, so it has no raster here. resvg is also what tty7 uses
to draw the tray icon, so these PNGs match what the app renders. Every size is
rendered from the vector instead of downscaled from 1024, which keeps the
prompt crisp in the 16-32 px frames Windows shows most.
The social card keeps its existing layout; its 141 px logo tile is replaced
on the card's solid background each time, so repeated renders do not stack.
"""

import shutil
from io import BytesIO
from pathlib import Path

import icnsutil
import resvg_py
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / "assets"


def render(svg: str, size: int) -> Image.Image:
    png = resvg_py.svg_to_bytes(svg_path=str(ASSETS / svg), width=size, height=size)
    return Image.open(BytesIO(png)).convert("RGBA")


def png_bytes(image: Image.Image) -> bytes:
    out = BytesIO()
    image.save(out, "PNG", optimize=True)
    return out.getvalue()


def main() -> None:
    render("app-icon.svg", 1024).save(ASSETS / "app-icon.png", optimize=True)
    render("logo.svg", 1024).save(ASSETS / "logo.png", optimize=True)
    render("logo.svg", 256).save(ASSETS / "logo@256.png", optimize=True)

    with Image.open(ASSETS / "social-preview.png") as card:
        card = card.convert("RGB")
    tile = Image.new("RGBA", (141, 141), "#0e0e11")
    tile.alpha_composite(render("logo.svg", 141))
    card.paste(tile.convert("RGB"), (170, 140))
    card.save(ASSETS / "social-preview.png", optimize=True)

    # Embedded in the .exe by build.rs, and the installer's icon. Pillow only
    # writes frames no larger than the image it saves, so 256 goes first.
    frames = [render("app-icon.svg", s) for s in (256, 128, 64, 48, 32, 24, 16)]
    frames[0].save(
        ASSETS / "favicon.ico",
        sizes=[f.size for f in frames],
        append_images=frames[1:],
    )

    # The docs site keeps its own copies.
    shutil.copyfile(ASSETS / "favicon.ico", ROOT / "docs" / "favicon.ico")
    shutil.copyfile(ASSETS / "logo.svg", ROOT / "docs" / "logo" / "logo.svg")

    # The entries iconutil writes: 16 and 32 at 1x are ARGB, the rest PNG.
    icns = icnsutil.IcnsFile()
    for key, size in [("ic04", 16), ("ic05", 32)]:
        argb = icnsutil.ArgbImage(image=render("app-icon.svg", size)).argb_data()
        icns.add_media(key, data=argb)
    for key, size in [
        ("ic11", 32),
        ("ic12", 64),
        ("ic07", 128),
        ("ic13", 256),
        ("ic08", 256),
        ("ic14", 512),
        ("ic09", 512),
        ("ic10", 1024),
    ]:
        icns.add_media(key, data=png_bytes(render("app-icon.svg", size)))
    icns.write(str(ASSETS / "tty7.icns"))


if __name__ == "__main__":
    main()
