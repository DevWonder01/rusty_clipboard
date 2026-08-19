import shutil
import os

src = "/home/arcbase/.gemini/antigravity/brain/a4ef319b-fb3f-498a-8dac-059ad02cb3be/media__1787125754805.png"
dst = "/home/arcbase/Documents/Github/OpenSource/rusty_clipboard/assets/screenshot.png"

if os.path.exists(src):
    shutil.copyfile(src, dst)
    print("Copied successfully to assets/screenshot.png")
