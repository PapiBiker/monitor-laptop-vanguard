# Monitor Rust para Windows

Panel compacto (264 x 262 px a 100 % de escala) integrado en la capa del escritorio de Windows, en la esquina superior derecha y detras de las ventanas normales. Muestra CPU con grafico del ultimo minuto, RAM, espacio libre en el disco del sistema, velocidad de red, numero de procesos y tiempo encendido.

El estilo acompana el fondo corporativo: gris pizarra (`#3C4650`) con tipografia blanca y una marca en V. Los colores estan en las constantes `COLOR_*` de `src/main.rs`. Las cifras cambian a ambar cuando la CPU pasa de 85 % o la RAM o el disco pasan de 90 %.

El programa usa directamente APIs Win32/GDI y no tiene dependencias Rust externas. Las metricas se actualizan cada segundo (el disco cada 15 s). Consumo medido: unos 3 MB de RAM. Se adapta al escalado de pantalla (DPI), solo permite una instancia y vuelve a aparecer si el Explorador de Windows se reinicia.

## Compilar y probar

Desde PowerShell:

```powershell
$env:Path = "$HOME\.cargo\bin;$env:Path"
cd .\monitor-rust
cargo fmt --check
cargo test
cargo build --release
```

El ejecutable queda en `target\release\monitor-rust.exe`.

## Ejecutar

```powershell
.\target\release\monitor-rust.exe
```

El panel no lanza ni ejecuta programas: solamente consulta y presenta metricas locales.

## Inicio de Windows

Para registrar el ejecutable actual en el inicio de sesion del usuario:

```powershell
.\target\release\monitor-rust.exe --install-startup
```

Para quitarlo del inicio:

```powershell
.\target\release\monitor-rust.exe --remove-startup
```

Si mueves o recompilas el ejecutable en otra ruta, vuelve a ejecutar `--install-startup`.

La temperatura de CPU/GPU no se muestra: Windows no ofrece una API universal para leerla en todas las computadoras.
