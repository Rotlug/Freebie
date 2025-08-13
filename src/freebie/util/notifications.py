from gi.repository import Gio, Gtk

from freebie.backend.game import Game


class Notifications:
    @staticmethod
    def send(application: Gtk.Application, notif: Gio.Notification):
        application.send_notification(None, notif)

    @staticmethod
    def install_finished(game: Game, application: Gtk.Application):
        notif = Gio.Notification.new(f"{game.name} has finished installing!")
        notif.set_body(f"{game.name} is now ready to play!")

        Notifications.send(application, notif)

    @staticmethod
    def install_failed(game: Game, application: Gtk.Application):
        notif = Gio.Notification.new(f"{game.name} failed to install!")
        notif.set_body(f"Something went wrong while installing {game.name}")

        Notifications.send(application, notif)
