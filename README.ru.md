<div align="center">

<img src="src-tauri/icons/128x128.png" width="92" alt="Airo" />

# Airo

**Свой цвет выделения для каждого монитора в Windows.**
Задайте каждому экрану собственный цвет — Airo переключает его сам, когда курсор переходит на другой монитор.

[English](README.md) · Русский

[![Release](https://img.shields.io/github/v/release/Toveal/airo?style=flat-square&color=4a6cf7)](https://github.com/Toveal/airo/releases)
[![Downloads](https://img.shields.io/github/downloads/Toveal/airo/total?style=flat-square&color=f5a524)](https://github.com/Toveal/airo/releases)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?style=flat-square)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB?style=flat-square)
[![License: MIT](https://img.shields.io/github/license/Toveal/airo?style=flat-square&color=97ca00)](LICENSE)

</div>

<!--
  ДЕМО. Тег <video> подтягивает закоммиченный MP4 по raw-ссылке из ветки `main` —
  файл docs/demo.mp4 должен быть запушен туда. Если у зрителя плеер не покажется,
  запасная ссылка ниже всё равно откроет клип. Самый надёжный способ: открыть этот
  README в редакторе на github.com и перетащить docs/demo.mp4 — GitHub вставит
  user-attachments-ссылку, которая встраивается всегда.
-->
<div align="center">

<video src="https://github.com/Toveal/airo/raw/main/docs/demo.mp4" width="760" autoplay loop muted playsinline controls></video>

[▶ Смотреть демо](https://github.com/Toveal/airo/raw/main/docs/demo.mp4)

<sub><i>Свой цвет выделения на каждом мониторе — Airo переключает его вслед за курсором.</i></sub>

</div>

## Что это

Windows красит рамку выделения — ту, что вы тянете мышью по рабочему столу или в проводнике — одним цветом на всех экранах. Airo позволяет задать **свой цвет для каждого монитора** и применяет его автоматически, как только курсор переходит на другой экран. Работает тихо в трее и запускается вместе с Windows.

## Возможности

- 🎨 **Цвет на каждый монитор** — контур и заливка, отдельно для каждого экрана
- 🖱️ **Авто-переключение** — следует за курсором между мониторами в реальном времени
- 🌗 **Светлая и тёмная тема**
- 🌐 **English и Русский** — и окно, и меню в трее
- 🚀 **Автозапуск и трей** — стартует свёрнутым, не мешает
- ⬇️ **Авто-обновление** — проверяет релизы GitHub и обновляется в один клик
- 🔒 **Без прав администратора** — установка per-user, без запроса UAC

## Скриншоты

<table align="center">
  <tr>
    <td width="50%" valign="top">
      <img src="docs/window-ru.png" alt="Главное окно Airo" /><br/>
      <sub><i>Цвета контура и заливки для каждого монитора, с живым превью.</i></sub>
    </td>
    <td width="50%" valign="top">
      <img src="docs/settings-ru.png" alt="Настройки Airo" /><br/>
      <sub><i>Автозапуск, старт свёрнутым, поведение курсора и язык.</i></sub>
    </td>
  </tr>
</table>

## Установка

1. Скачайте **`Airo_x.y.z_x64-setup.exe`** из [последнего релиза](https://github.com/Toveal/airo/releases/latest).
2. Запустите. Установщик **per-user** — ставит в `%LOCALAPPDATA%\Airo` без запроса прав администратора.
3. При первом запуске Windows SmartScreen может предупредить о неизвестном издателе (приложение не подписано платным сертификатом). Нажмите **Подробнее → Выполнить в любом случае**.

## Как пользоваться

1. Выберите вкладку монитора и задайте цвета **контура** и **заливки**.
2. Закройте окно — оно свернётся в трей и продолжит работать.
3. Двигайте курсор между экранами; цвет выделения следует за ним.

Всё сохраняется на лету. В **Настройках** — автозапуск, старт свёрнутым, смена цвета по клику и язык интерфейса.

## Обновления

Airo проверяет релизы GitHub при запуске. Когда выходит новая версия, внизу окна появляется баннер с кнопкой **«Загрузить и установить»** — один клик обновляет и перезапускает приложение.

## Сборка из исходников

**Требования:** [Rust](https://www.rust-lang.org/tools/install) (stable) · [Bun](https://bun.sh) · Windows 10/11

```bash
bun install
bun run tauri dev      # запуск в разработке
bun run tauri build    # сборка установщика
```

> **Важно.** Релизная сборка подписывает артефакт обновления, поэтому `tauri build` требует ключ подписи в переменных окружения:
>
> ```powershell
> $env:TAURI_SIGNING_PRIVATE_KEY = "<ваш приватный ключ>"
> $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
> bun run tauri build
> ```
>
> Для `bun run tauri dev` это не нужно.

**Стек:** Rust · Tauri 2 · Vue 3 · Vite

## Лицензия

Распространяется под [лицензией MIT](LICENSE) © 2026 Toveal.
