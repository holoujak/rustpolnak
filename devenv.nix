{ pkgs, lib, config, ... }: {
	languages.rust.enable = true;
	languages.python.enable = true;
	languages.python.uv.enable = true;
	packages = [
		pkgs.dioxus-cli
		pkgs.glib
		pkgs.gtk3
		pkgs.webkitgtk_4_1
		pkgs.xdotool
	];
	processes = {
		uv.exec = "uv --directory stubs run fastapi dev";
		uv2.exec = "uv --directory stubs run rfid.py";
	};
	enterShell = "rustc --version;cargo --version;uv --version";
}
