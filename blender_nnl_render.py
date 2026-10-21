"""Headless Blender renderer for ffplayout NNL dynamic graphics.

Server-side helper: renders the animated NOW / NEXT / LATER / Current-Movie
graphic for a single movie title, producing a transparent (straight-alpha RGBA)
PNG sequence that server.py transcodes into a cached alpha ProRes 4444 MOV.

Usage (called by server.py):
    blender -b <scene.blend> -y --python blender_nnl_render.py -- \
        --title "Movie Title" --out C:/tmp/nnl_blend_xxx \
        --fps 25 --frames 125 [--font C:/Windows/Fonts/arial.ttf]

Scene authoring contract:
  * The scene must contain exactly the title as a Blender Text object, OR an
    object whose name contains "title", OR a scene custom string property
    ``NNL_TITLE_OBJECT`` naming the text object. The text body is replaced with
    the movie title and the text object should be laid out/centered on screen.
  * Film > Transparent is forced ON and output is PNG RGBA at the scene's
    resolution and fps (all set by this script; scene resolution + fps are
    read by the server for the final encode).
  * Keyframe the animation (slide/fade/scale…) so it loops or holds cleanly.
"""
import argparse
import sys


def main() -> int:
    parser = argparse.ArgumentParser(description="Render one NNL movie graphic.")
    parser.add_argument("--title", default="", help="Movie title (Line1)")
    parser.add_argument("--out", required=True, help="Output directory")
    parser.add_argument("--fps", type=float, default=25.0)
    parser.add_argument("--frames", type=int, default=25)
    parser.add_argument("--font", default="", help="Font file for non-Latin text")
    args, _ = parser.parse_known_args()

    import bpy  # only available inside Blender's bundled python

    scene = bpy.context.scene

    title_obj = None
    if "NNL_TITLE_OBJECT" in scene.keys():
        name = scene["NNL_TITLE_OBJECT"]
        if name in bpy.data.objects:
            title_obj = bpy.data.objects[name]
    if title_obj is None:
        fonts = [o for o in bpy.data.objects if o.type == "FONT"]
        for o in fonts:
            if "title" in o.name.lower():
                title_obj = o
                break
    if title_obj is None and len(fonts) == 1:
        title_obj = fonts[0]
    if title_obj is None:
        print("NNLERR no title text object found in scene", file=sys.stderr)
        return 2
    if title_obj.type != "FONT":
        print(f"NNLERR title object '{title_obj.name}' is not a text object",
              file=sys.stderr)
        return 3

    title_obj.data.body = args.title
    if args.font:
        try:
            title_obj.data.font = bpy.data.fonts.load(args.font)
        except Exception:
            try:
                title_obj.data.font = bpy.data.fonts.load(
                    args.font.replace("\\", "/"))
            except Exception:
                pass

    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.resolution_percentage = 100
    scene.frame_start = 1
    scene.frame_end = max(1, args.frames)
    scene.render.fps = max(1, int(round(args.fps)))
    scene.render.fps_base = 1
    scene.render.filepath = args.out.rstrip("/\\") + "/frame_"

    bpy.ops.render.render(animation=True)
    print(f"NNLDONE rendered {args.frames} frames")
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)