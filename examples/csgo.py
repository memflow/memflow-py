"""
This is a crude example of a game hack using memflow-py
Run using the `native` connector on a windows PC, OR tear out the local window and maybe communicate to target pc via shared memory. 
"""
import pyglet

# NOTE: We do NOT touch the game process using either of these, just for making an overlay
import win32gui, win32con

from struct import unpack
from memflow import *
from ctypes import *
from pyglet.window import Window

GAME_WINDOW_POS = [0, 0]
GAME_WINDOW_SIZE = [3440.0, 1440.0]
OFFSET_ENTITY_LIST = 0x4DFFF14
OFFSET_LOCAL_PLAYER = 0xDEA964
OFFSET_VIEW_MATRIX = 0x4DF0D44
MAX_PLAYERS = 32

inventory = Inventory()
os = inventory.os("native")
process = os.process_from_name("csgo.exe")
client_mod_addr = process.module_from_name("client.dll").base


class Vec3(Structure):
    _fields_ = [("bytes", c_uint32 * 3)]

    def to_float(self):
        return unpack("3f", self.bytes)

    def __str__(self):
        return f"{self.to_float()}"


class Matrix4X4(Structure):
    _fields_ = [
        ("f0", c_uint32 * 4),
        ("f1", c_uint32 * 4),
        ("f2", c_uint32 * 4),
        ("f3", c_uint32 * 4),
    ]

    def to_float(self):
        return [
            unpack("4f", self.f0),
            unpack("4f", self.f1),
            unpack("4f", self.f2),
            unpack("4f", self.f3),
        ]

    def __str__(self):
        return f"{self.to_float()}"


class Entity(Structure):
    _fields_ = [
        ("_pad0x0", c_uint8 * 0xAC),
        ("m_vecOrigin", Vec3),
        ("_pad0xB8", c_uint8 * 0x3C),
        ("m_iTeamNum", c_uint32),
        ("_pad0xf8", c_uint8 * 8),
        ("m_iHealth", c_int32),
    ]

    def __str__(self):
        return f"Entity ({self.m_iHealth} HP) at {self.m_vecOrigin}, on team {self.m_iTeamNum}"


def read_entities():
    entity_ptrs = process.read(
        client_mod_addr + OFFSET_ENTITY_LIST, POINTER32(Entity) * (4 * MAX_PLAYERS)
    )
    entities = []
    it = iter(entity_ptrs)
    for ptr in it:
        if ptr.addr == 0:
            break
        entities.append(process.read_ptr(ptr))
        drop = next(it)  # Skips the other stuff in between
        drop = next(it)
        drop = next(it)
    return entities


def world_to_screen(screen_size, vm, pos):
    clip_x = (pos[0] * vm[0][0]) + (pos[1] * vm[0][1]) + (pos[2] * vm[0][2]) + vm[0][3]
    clip_y = (pos[0] * vm[1][0]) + (pos[1] * vm[1][1]) + (pos[2] * vm[1][2]) + vm[1][3]
    clip_w = (pos[0] * vm[3][0]) + (pos[1] * vm[3][1]) + (pos[2] * vm[3][2]) + vm[3][3]
    if clip_w < 0.1:
        return False
    ndc_x = clip_x / clip_w
    ndc_y = clip_y / clip_w
    screen_x = (screen_size[0] / 2 * ndc_x) + (ndc_x + screen_size[0] / 2)
    screen_y = (screen_size[1] / 2 * ndc_y) + (ndc_y + screen_size[1] / 2)
    return [screen_x, screen_y]


# I subtract a bit here to prevent window automatic hdr nonsense, u can remove if u dont have hdr enabled
window = Window(
    int(GAME_WINDOW_SIZE[0]) - 40,
    int(GAME_WINDOW_SIZE[1]) - 30,
    style=Window.WINDOW_STYLE_OVERLAY,
)
window.set_caption("")
# Correct `_hwnd`.
win32gui.SetLayeredWindowAttributes(window._hwnd, 0, 0, win32con.LWA_COLORKEY)
# Correct `_view_hwnd`.
win32gui.SetWindowLong(
    window._view_hwnd,
    win32con.GWL_EXSTYLE,
    win32gui.GetWindowLong(window._view_hwnd, win32con.GWL_EXSTYLE)
    | win32con.WS_EX_LAYERED,
)
win32gui.SetLayeredWindowAttributes(window._view_hwnd, 0, 0, win32con.LWA_COLORKEY)


@window.event
def on_draw():
    window.clear()
    lp_ptr = process.read(client_mod_addr + OFFSET_LOCAL_PLAYER, POINTER32(Entity))
    # Make sure we are in game.
    if lp_ptr.addr == 0:
        return
    local_player = process.read_ptr(
        process.read(client_mod_addr + OFFSET_LOCAL_PLAYER, POINTER32(Entity))
    )
    view_matrix = process.read(client_mod_addr + OFFSET_VIEW_MATRIX, Matrix4X4)
    entities = read_entities()
    for entity in entities:
        if entity.m_iTeamNum == local_player.m_iTeamNum:
            continue
        if entity.m_iHealth <= 0:
            continue
        # Entity is enemy and alive, show health at feet
        w2s_origin = world_to_screen(
            GAME_WINDOW_SIZE,
            view_matrix.to_float(),
            entity.m_vecOrigin.to_float(),
        )
        if w2s_origin:
            pyglet.text.Label(
                f"{entity.m_iHealth} HP",
                font_name="Consolas",
                font_size=12,
                x=int(w2s_origin[0]),
                y=int(w2s_origin[1]),
                anchor_x="center",
                anchor_y="top",  # Top is actually the opposite from traditional positioning (its bottom if you are use to top down)
            ).draw()


pyglet.app.run()
