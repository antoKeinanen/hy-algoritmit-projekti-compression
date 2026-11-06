# Compression explorer

- Ajattelin käyttää rustia projektin ytimeen ja visualisointiin typescriptiä ja reactia.
- Kielet joita osaan tarpeeksi sujuvasti: typescript/javascript, python, c/c++.
- Ajattelin toteuttaa seuraavat pakkaus algoritmit: Run-Length Encoding, Huffman Coding, LZ77
    - Jos aikaa jää olisi kiva toteuttaa: LZW, deflate
- En välttämättä ratkaise mitään ongelmaa, mutta mielestäni olisi mielenkiintoista rakentaa pieni nettisivu, johon voi syöttää jonkin syötteen ja se esittelisi jotenkin visuaalisesti eri pakkaus algoritmejä ja niiden lopputuloksia.


## Aikavaativuudet

| Algoritmi | Pakkaaminen | Purkaminen |
|---|---|---|
| Run-Length Encoding (RLE) | $O(n)$ | $O(m)$ |
| Huffman Coding | $O(n + k\log k)$ | $O(m)$ |
| LZ77 | $O(nw)$, pahimmillaan $O(n^2)$ | $O(m)$ |
| LZW | $O(n)$ keskimäärin | $O(m)$ |
| DEFLATE | $O(nw + k\log k)$ | $O(m)$ |

- $n$ on alkuperäisen syötteen koko
- $m$ on pakatun datan koko
- $k$ on erilaisten symbolien määrä
- $w$ on hakuikkunan koko

## Lähteet
- https://en.wikipedia.org/wiki/Run-length_encoding
- https://www.geeksforgeeks.org/dsa/huffman-coding-greedy-algo-3/
- https://en.wikipedia.org/wiki/LZ77_and_LZ78
- https://en.wikipedia.org/wiki/Lempel–Ziv–Welch
- https://zlib.net/feldspar.html


## Projektin ydin
Projektin ydin tulee olemaan wasm käännetty rust kirjasto, joka toteuttaa muutaman pakkaus ja purku algoritmin. Lisä ominaisuutena luon kivan käyttöliittymän, jossa voi tutustua ja vertailla näitä algoritmejä.


---

*Opiskelen tietojenkäsittelytieteen kandidaatti-ohjelmassa*
