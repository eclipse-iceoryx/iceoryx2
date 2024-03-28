mkdir "C:\Temp\iceoryx2\services"
mkdir "C:\Temp\iceoryx2\tests"
mkdir "C:\Temp\iceoryx2\shm"
icacls "C:\Temp" /t /c /grant Everyone:F
setx RUSTFLAGS "-C debug-assertions"