from kivy.uix.screenmanager import Screen
from kivy.uix.floatlayout import FloatLayout
from kivymd.uix.label import MDLabel

from kivy.clock import Clock, mainthread

from kivy.properties import ObjectProperty, DictProperty
from kivy.uix.boxlayout import BoxLayout

from kivy.uix.widget import Widget

from kivy.uix.image import Image
from kivy.uix.floatlayout import FloatLayout
from kivymd.uix.label import MDLabel
from kivymd.uix.slider import MDSlider
from kivy.uix.scrollview import ScrollView

from kivymd.uix.toolbar import MDTopAppBar
from kivymd.uix.boxlayout import MDBoxLayout
from kivymd.uix.gridlayout import MDGridLayout

from kivy.graphics.texture import Texture
from kivy.graphics import Rectangle, Color

from kivymd.theming import ThemeManager

from .utils import *

THEMECOLORS = ThemeManager()
THEMECOLORS.theme_style = "Dark"


class Slider(MDSlider):
    def __init__(self, **kwargs):
        if "id" in kwargs.keys():
            id = kwargs.get("id")
            instance: EditorSheet = kwargs.get("instance")
            instance.sliders_control[id] = self
            kwargs.pop("id")
            kwargs.pop("instance")

        super().__init__(**kwargs)

    """def collide_point(self, x, y):
        return self.x <= x <= self.right and self.y + 250 <= y <= self.top - 250"""


class Spacer(MDBoxLayout):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)

        if kwargs.get("orientation") == "horizontal":
            self.size_hint_y = None
            self.height = 5
        else:
            self.size_hint_x = None
            self.width = 5

        self.md_bg_color = tuple(THEMECOLORS.bg_darkest)


class PPMImage(Widget):
    tex = ObjectProperty(None)

    def __init__(self, file, exposition=0, **kwargs):
        super().__init__(**kwargs)

        self.data = open(file, "r").read().split("\n")

        self.magic_number = self.data[0]

        self.size = tuple([int(s) for s in self.data[1].strip().split(" ")])

        max_val = int(self.data[2])

        Clock.schedule_once(lambda instance: self.texture_init(exposition), 0)

    def texture_init(self, exposition):
        self.tex = Texture.create(size=self.size)

        data = self.data[3:]
        data.reverse()

        buf = []

        for d in data:
            if d != "":
                buf += [
                    (int(s) + exposition)
                    if 0 <= (int(s) + exposition) < 256
                    else (255 if exposition > 0 else 0)
                    for s in [d.split(" ")[0], d.split(" ")[1], d.split(" ")[2]]
                ]

        self.buffer = bytes(buf)

        self.tex.add_reload_observer(self.populate_texture)

        with self.canvas:
            self.rect = Rectangle(texture=self.tex, pos=self.pos, size=self.size)

        self.populate_texture(self.tex)

    def populate_texture(self, texture):
        texture.blit_buffer(self.buffer, colorfmt="rgb", bufferfmt="ubyte")


class ColorManager(object):
    def __init__(self, buffer=None, format="rgb") -> None:
        self.buffer = buffer
        self.format = format

    def RGB_from_RGBA(buffer):
        new_buffer = []
        for i in range((buffer.__len__() / 4) + 1):
            new_buffer += [buffer[i : i + 3]]
        return ColorManager(new_buffer)

    def HSV_from_RGB(self, buffer=None):
        new_buffer = []
        if not buffer:
            buffer = self.buffer
        for i in range((buffer.__len__() / 3) + 1):
            new_buffer += rgb_to_hsv(*buffer[i : i + 3])
        return ColorManager(new_buffer, format="hsv")

    def RGB_from_HSV(self, buffer=None):
        if not buffer:
            buffer = self.buffer
        new_buffer = []
        for i in range((buffer.__len__() / 3) + 1):
            new_buffer += hsv_to_rgb(*buffer[i : i + 3])
        return ColorManager(new_buffer, format="rgb")


class EditorSheet(Screen):
    sliders_control = DictProperty({})

    def __init__(self, **kw):
        super().__init__(**kw)
        self.name = "main"

        frame = MDBoxLayout(orientation="vertical")

        toolbar = MDTopAppBar()
        toolbar.title = "Kivy Photo Editor"

        toolbar.left_action_items = [["plus", self.open_file]]

        frame.add_widget(toolbar)

        subframe = BoxLayout()

        vtoolbar = MDBoxLayout(orientation="vertical")
        vtoolbar.size_hint_x = None
        vtoolbar.width = 100
        vtoolbar.md_bg_color = tuple(THEMECOLORS.bg_dark)
        subframe.add_widget(vtoolbar)

        subframe.add_widget(Spacer())

        box = MDBoxLayout()
        box.md_bg_color = tuple(THEMECOLORS.bg_dark)
        self.image_loader = Image()
        box.add_widget(self.image_loader)
        subframe.add_widget(box)

        subframe.add_widget(Spacer())

        subframe.add_widget(self.create_slider_space())

        frame.add_widget(subframe)

        self.add_widget(frame)

        self.fl = FloatLayout()
        self.fl.size_hint = (None, None)
        self.fl.pos = (0, 0)

        label = MDLabel(text="Waiting for the file", halign="center")
        label.pos_hint = {"center_x": 0.5, "center_y": 0.5}

        self.fl.add_widget(label)

    def create_slider_space(self) -> MDBoxLayout:
        sliders_space = MDBoxLayout(orientation="vertical")
        sliders_space.md_bg_color = tuple(THEMECOLORS.bg_dark)
        sliders_space.size_hint_x = None
        sliders_space.width = 500

        sv = ScrollView(do_scroll_x=False)

        fields = ["Exposition", "Saturation", "Contrast"]

        sv.add_widget(
            MDBoxLayout(
                *(
                    MDGridLayout(
                        MDLabel(text=f),
                        Slider(id=f.lower(), instance=self),
                        # adaptive_height=True,
                        cols=1,
                        rows=2,
                        padding=[20, 5, 20, 5],
                    )
                    for f in fields
                ),
                orientation="vertical"
            )
        )

        sliders_space.add_widget(sv)

        return sliders_space

    @mainthread
    def open_file(self, instance):
        with self.canvas:
            Color(0, 0, 0, 0.85)
            self.rect = Rectangle(pos=(0, 0), size=self.size)
            self.fl.size = self.size
            self.add_widget(self.fl)

        def _open_file(*args):
            from tkinter import Tk
            from tkinter.filedialog import askopenfilename

            Tk().withdraw()

            f = askopenfilename(filetypes=[("Image file", "*.jpg")])

            self.image_loader.source = f

            self.remove_widget(self.fl)
            self.canvas.remove(self.rect)

        Clock.schedule_once(_open_file, 1)

    def connect_sliders(self):
        self.sliders_control["saturation"].bind(value=self.Saturation)

    def Saturation(self, value):
        cm = ColorManager.RGB_from_RGBA(self.image_loader.texture.pixels)
        hsv = cm.HSV_from_RGB()

        for i in range((hsv.__len__() / 3) + 1):
            hsv[i + 2] = hsv[i + 2] - 0.2 if hsv[i + 2] > 0.2 else hsv[i + 2]


class Loading(Screen):
    def __init__(self, **kw):
        super().__init__(**kw)

        self.name = "loading"
        from kivymd.uix.spinner import MDSpinner
        from kivy.metrics import dp

        box = FloatLayout()
        lbl = MDLabel(text="Loading...", halign="center")
        lbl.pos_hint = {"center_x": 0.5, "center_y": 0.55}
        box.add_widget(lbl)
        box.add_widget(
            MDSpinner(
                size_hint=(None, None),
                size=(dp(46), dp(46)),
                pos_hint={"center_x": 0.5, "center_y": 0.45},
                active=True,
                palette=[
                    [0.28627450980392155, 0.8431372549019608, 0.596078431372549, 1],
                    [0.3568627450980392, 0.3215686274509804, 0.8666666666666667, 1],
                    [0.8862745098039215, 0.36470588235294116, 0.592156862745098, 1],
                    [
                        0.8784313725490196,
                        0.9058823529411765,
                        0.40784313725490196,
                        1,
                    ],
                ],
            )
        )
        self.add_widget(box)
