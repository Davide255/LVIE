from kivymd.app import MDApp
from .app_pages import *
from kivy.uix.screenmanager import ScreenManager
from kivy.core.window import Window
from kivy.clock import Clock, mainthread
from kivymd.theming import ThemeManager

from .utils import *


class PhotoEditor(MDApp):
    def build(self):
        Window.maximize()

        self.sm = ScreenManager()

        self.sm.add_widget(Loading())
        self.sm.bind(
            current=lambda *args: setattr(self.theme_cls, "theme_style", "Dark")
        )

        self.theme_cls.theme_style = "Dark"

        Clock.schedule_once(self.start_loading, 1)
        return self.sm

    def start_loading(self, *args):
        self.sm.add_widget(EditorSheet())
        self.theme_cls.theme_style = "Dark"
        self.sm.current = "main"
