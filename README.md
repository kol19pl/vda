vda – Video Download Assistant (Rust Edition)

Opis:  
vda to port oryginalnego projektu autora *Slayter* (Python) do Rust, stworzony z myślą o stabilnym działaniu na starszym sprzęcie i systemach takich jak Synology DSM. Projekt rozwija funkcjonalność wersji bazowej 2.0 i umożliwia pracę 24h na różnych platformach.

Co nowego:  
- Możliwość zmiany portu serwera i działania nie tylko na localhost.  
  - Jeśli wybrany port jest zajęty, serwer automatycznie próbuje kolejne porty aż do 10 prób.  
- Obsługa yt-dlp w safe-container, działa niezależnie od wersji Pythona w systemie, ale jeśli Python i yt-dlp są obecne, użyje ich.  
- Wsparcie dla FFmpeg z dodatkowymi metodami, aby odnaleźć stare wersje w systemie (np. na Synology DSM).  
- Binarki przygotowane dla wielu systemów, w tym Linux MUSL (Synology ARM/Intel), Windows (MSVC/GNU), macOS, z nazwami platform w pliku.

Obsługiwane systemy (testowane przez autora):  
- Windows 11 (aktywnie używany)  
- Windows 10 (GNU, testowane)  
- Linux MUSL (aktywnie używany)  
- Synology DSM 7 (aktywnie używany)  

> Inne platformy mogą być dostępne, ale nie są testowane.

Aktułanie pracuje nad gui pod synology oraz przepisaniem pluginu aby mugł sterować serwerem z jego pozimu

