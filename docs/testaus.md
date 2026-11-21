**Yksikkötestauksen kattavuusraportti.**

| Filename          | Regions | Missed Regions | Cover      | Functions | Missed Functions | Executed   | Lines   | Missed Lines | Cover      | Branches | Missed Branches | Cover |
|-------------------|---------|----------------|------------|-----------|------------------|------------|---------|--------------|------------|----------|-----------------|-------|
| lz77/src/lib.rs   | 187     | 0              | 100.00%    | 6         | 0                | 100.00%    | 106     | 0            | 100.00%    | 0        | 0               | -     |
| lzw/src/lib.rs    | 397     | 0              | 100.00%    | 22        | 0                | 100.00%    | 194     | 0            | 100.00%    | 0        | 0               | -     |
| shared/src/lib.rs | 178     | 3              | 98.31%     | 19        | 1                | 94.74%     | 115     | 3            | 97.39%     | 0        | 0               | -     |
| wasm/src/lib.rs   | 216     | 12             | 94.44%     | 26        | 2                | 92.31%     | 122     | 8            | 93.44%     | 0        | 0               | -     |
| **TOTAL**         | **978** | **15**         | **98.47%** | **73**    | **3**            | **95.89%** | **537** | **11**       | **97.95%** | **0**    | **0**           | -     |

**Mitä on testattu, miten tämä tehtiin?**

Testaan ohjelmaa toistaiseksi yksinkertaisilla testeillä. Tarkistan, että ohjelma pystyy pakkaamaan ja purkamaan syötteen. Testaus on toteutettu rustin sisäsillä testaus työkaluilla.
Testaan myös, että tuotetut välit ovat oikein ja kattavat koko syötteen.

**Minkälaisilla syötteillä testaus tehtiin?**

Testaus on tehty yksinkertaisilla binääri syötteillä.

**Miten testit voidaan toistaa?**

`readme.md` tiedostossa on ohjeet automaattisten testien ajamiseen.

**Ohjelman toiminnan mahdollisen empiirisen testauksen tulosten esittäminen graafisessa muodossa. (Mikäli sopii aiheeseen)**

Ohjelman käyttöliittymä on vielä niin yksinkertainen, että en näe mitään järkeä testata sitä vielä. Kutsun ainoastaan jo testattuja funktioita.

**Ei siis riitä todeta, että testaus on tehty käyttäen automaattisia yksikkötestejä, vaan tarvitaan konkreettista tietoa testeistä, kuten:**

Shared:
- Testaan bittien kirjoittamista ja lukemista eri bittipituuksilla. Tarkistan myös, että bittien järjestys vastaa bitvec-kirjaston käyttämää järjestystä.
- Testaan, että tuotetut välit ovat oikein ja että ne löytyvät oikeilla indekseillä. Tarkistan myös, että välit kattavat koko syötteen ja että virheelliset välit tunnistetaan.

LZ77:
- Testaan, että 24-bittinen syöte voidaan pakata ja purkaa takaisin alkuperäiseksi.
- Testaan myös syötteellä 10101011, että tuotetut välit vastaavat odotettuja arvoja. Tarkistan, että purkamisen välit vastaavat pakkaamisen välejä käänteisesti.
- Testaan myös tyhjän syötteen käsittelyn.

LZW:
- Testaan, että 24-bittinen syöte voidaan pakata ja purkaa takaisin alkuperäiseksi. Testaan myös tyhjää syötettä, yksittäisiä bittejä ja 1000 peräkkäistä ykköstä.
- Testaan, että pakkaaminen ja purkaminen toimii myös sanakirjan täyttyessä. Tarkistan myös, että toistuva syöte abcabcabc 100 kertaa pakkautuu alkuperäistä pienemmäksi.
- Testaan koodien bittipituuksien laskemista eri sanakirjakooilla.
- Testaan syötteellä 0000, että tuotetut bitit ja välit ovat oikein. Tarkistan myös neljällä eri syötteellä, että välit kattavat koko syötteen ja vastaavat purkamisen välejä käänteisesti.
- Testaan myös tyhjän syötteen välit sekä virheellisten ja kesken loppuvien pakattujen syötteiden käsittelyn.


WASM:
- Testaan, että LZ77 ja LZW pakkaaminen ja purkaminen toimivat myös WASM-rajapinnan kautta.
- Testaan, että pakattu data voidaan muuttaa tavuiksi ja bittipituudeksi sekä palauttaa takaisin ilman, että purkaminen epäonnistuu.
- Tarkistan myös, että rajapinnan palauttamat välit kattavat koko tulosteen ja vastaavat purkamisen välejä käänteisesti.
