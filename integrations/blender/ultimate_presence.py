# UltimatePresence for Blender
# Sends what you're doing in Blender (file, mode, workspace, render progress) to the
# UltimatePresence app on this PC, which shows it on your Discord profile.
# Only talks to 127.0.0.1 (your own computer). Nothing is sent anywhere else.
#
# Install: Edit → Preferences → Add-ons → Install from Disk… → pick this file → enable it.

bl_info = {
    "name": "UltimatePresence",
    "author": "aqua",
    "version": (1, 0, 0),
    "blender": (2, 93, 0),
    "location": "Runs in the background",
    "description": "Shows what you're doing in Blender on Discord, through the UltimatePresence app",
    "category": "System",
}

import atexit
import json
import threading
import time
import urllib.request

import bpy
from bpy.app.handlers import persistent

URL = "http://127.0.0.1:4455/blender"
CHECK_EVERY = 1.0  # seconds between looks at Blender's state
HEARTBEAT = 10.0  # resend even without changes, so the app knows Blender is alive

_started = time.time()
_last = None
_last_sent = 0.0
_render = {"on": False, "frame": None, "frames_seen": 0}

_OBJECT_MODES = {
    "TEXTURE_PAINT": "PAINT_TEXTURE",
    "VERTEX_PAINT": "PAINT_VERTEX",
    "WEIGHT_PAINT": "PAINT_WEIGHT",
}


def _send(data, wait=False):
    def go():
        try:
            req = urllib.request.Request(
                URL,
                data=json.dumps(data).encode("utf-8"),
                headers={"Content-Type": "application/json"},
                method="POST",
            )
            urllib.request.urlopen(req, timeout=2).read()
        except Exception:
            pass  # app not running: try again next time

    if wait:
        go()
    else:
        threading.Thread(target=go, daemon=True).start()


def _mode(obj):
    try:
        mode = bpy.context.mode
        if mode:
            return mode
    except Exception:
        pass
    if obj is None:
        return "OBJECT"
    if obj.mode == "EDIT":
        return "EDIT_" + obj.type
    return _OBJECT_MODES.get(obj.mode, obj.mode)


def _snapshot():
    wm = bpy.context.window_manager
    win = bpy.context.window or (wm.windows[0] if wm and len(wm.windows) else None)
    screen = win.screen if win else None
    workspace = win.workspace.name if win and win.workspace else ""

    areas = []
    if screen:
        for area in screen.areas:
            name = area.type
            space = area.spaces.active
            if area.type == "IMAGE_EDITOR":
                name += ":" + str(getattr(space, "ui_mode", "") or getattr(space, "mode", ""))
            elif area.type == "NODE_EDITOR":
                name += ":" + str(getattr(space, "tree_type", ""))
            areas.append(name)

    view_layer = bpy.context.view_layer
    obj = view_layer.objects.active if view_layer else None
    scene = bpy.context.scene
    filepath = bpy.data.filepath

    data = {
        "version": bpy.app.version_string.split(" ")[0],
        "file": bpy.path.basename(filepath) if filepath else None,
        "dirty": bool(bpy.data.is_dirty),
        "workspace": workspace,
        "mode": _mode(obj),
        "object": obj.name if obj else None,
        "objectType": obj.type if obj else None,
        "areas": areas,
        "playing": bool(screen.is_animation_playing) if screen else False,
        "rendering": _render["on"],
        "frame": scene.frame_current if scene else None,
        "started": _started,
    }
    if _render["on"]:
        data["renderFrame"] = _render["frame"]
        # Only an animation render has a frame range worth showing.
        if _render["frames_seen"] > 1 and scene:
            data["renderStart"] = scene.frame_start
            data["renderEnd"] = scene.frame_end
    return data


def _tick():
    global _last, _last_sent
    try:
        data = _snapshot()
    except Exception:
        return CHECK_EVERY
    now = time.time()
    # "dirty" flips on every edit; it isn't worth an update by itself.
    comparable = {k: v for k, v in data.items() if k != "dirty"}
    if comparable != _last or now - _last_sent >= HEARTBEAT:
        _last = comparable
        _last_sent = now
        _send(data)
    return CHECK_EVERY


@persistent
def _render_init(scene, *args):
    _render.update(on=True, frame=scene.frame_current, frames_seen=0)
    _tick()


@persistent
def _render_pre(scene, *args):
    _render["frame"] = scene.frame_current
    _render["frames_seen"] += 1
    _tick()


@persistent
def _render_done(scene, *args):
    _render.update(on=False, frame=None, frames_seen=0)
    _tick()


def _closing():
    _send({"closing": True}, wait=True)


_HANDLERS = (
    ("render_init", _render_init),
    ("render_pre", _render_pre),
    ("render_complete", _render_done),
    ("render_cancel", _render_done),
)


def register():
    for name, fn in _HANDLERS:
        lst = getattr(bpy.app.handlers, name, None)
        if lst is not None and fn not in lst:
            lst.append(fn)
    if not bpy.app.timers.is_registered(_tick):
        bpy.app.timers.register(_tick, first_interval=1.0, persistent=True)
    atexit.register(_closing)


def unregister():
    for name, fn in _HANDLERS:
        lst = getattr(bpy.app.handlers, name, None)
        if lst is not None and fn in lst:
            lst.remove(fn)
    if bpy.app.timers.is_registered(_tick):
        bpy.app.timers.unregister(_tick)
    try:
        atexit.unregister(_closing)
    except Exception:
        pass
    _closing()


if __name__ == "__main__":
    register()
