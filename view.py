from core.Viewer import PPMImage, BoxLayout

from kivymd.app import MDApp


class App(MDApp):
    def build(self):
        # return PPMImage("D:\\Projects\\Ray tracing\\raytracer\\image.ppm")
        box = BoxLayout()
        box.add_widget(
            PPMImage("D:\\Projects\\Ray tracing\\raytracer\\first_image.ppm")
        )
        return box


App().run()
