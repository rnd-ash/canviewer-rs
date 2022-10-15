# canviewer-rs
A Realtime CAN network viewer with DBC support, powered by [ecu_diagnostics](https://github.com/rnd-ash/ecu_diagnostics) and [egui](https://github.com/emilk/egui)

<p align="center">
<img align="center" height="200" src="canviewer/logo.png">
</p>

## Checklist (What works and what doesn't)

- [x] DBC loading
- [x] Light / Dark theme
- [x] SocketCAN support
- [ ] Historical graphing of CAN data
- [x] CAN Frame viewer
- [ ] Appimage generation
- [ ] Editing of DBC files
- [ ] Showing signal sender and receivers as a node graph
- [ ] Signal / message searching

## Usage

You will need to first build canviewer-rs with rust before use!

```
./canviewer <SOCKETCAN IFACE> <DBC FILE>
```

EG:
```
./canviewer can0 my_can_dbc.dbc
```

## Images
<p align="center">
<img align="center" height="500" src="images/s1.png">
</p>