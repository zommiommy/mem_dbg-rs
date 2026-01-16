# Comparison Results

## Full Data

| Type   | Container | Crate    | Size     | Computed (bytes) | Error % | Time (ns) |
|--------|-----------|----------|----------|------------------|---------|-----------|
| String | BTreeMap  | deepsize | 0        | 24               | 0.0000  | 160       |
| String | BTreeMap  | get-size | 0        | 24               | 0.0000  | 30        |
| String | BTreeMap  | mem_dbg  | 0        | 24               | 0.0000  | 30        |
| String | BTreeMap  | deepsize | 10       | 573              | 22.2524 | 80        |
| String | BTreeMap  | get-size | 10       | 673              | 8.6839  | 60        |
| String | BTreeMap  | mem_dbg  | 10       | 698              | 5.2917  | 60        |
| String | BTreeMap  | deepsize | 100      | 14349            | 28.5586 | 220       |
| String | BTreeMap  | get-size | 100      | 15349            | 23.5798 | 221       |
| String | BTreeMap  | mem_dbg  | 100      | 19416            | 3.3308  | 220       |
| String | BTreeMap  | deepsize | 1000     | 145191           | 24.7649 | 1332      |
| String | BTreeMap  | get-size | 1000     | 155191           | 19.5831 | 1362      |
| String | BTreeMap  | mem_dbg  | 1000     | 199266           | 3.2557  | 1452      |
| String | BTreeMap  | deepsize | 10000    | 1471251          | 24.5439 | 18338     |
| String | BTreeMap  | get-size | 10000    | 1571251          | 19.4152 | 17807     |
| String | BTreeMap  | mem_dbg  | 10000    | 2015766          | 3.3826  | 22104     |
| String | BTreeMap  | deepsize | 100000   | 14909151         | 24.2900 | 370352    |
| String | BTreeMap  | get-size | 100000   | 15909151         | 19.2119 | 228456    |
| String | BTreeMap  | mem_dbg  | 100000   | 20360766         | 3.3938  | 601391    |
| String | BTreeMap  | deepsize | 1000000  | 151070151        | 24.0533 | 14935747  |
| String | BTreeMap  | get-size | 1000000  | 161070151        | 19.0260 | 14699930  |
| String | BTreeMap  | mem_dbg  | 1000000  | 205610766        | 3.3656  | 18098168  |
| String | BTreeMap  | deepsize | 10000000 | 1530590151       | 23.8146 | 180258225 |
| String | BTreeMap  | get-size | 10000000 | 1630590151       | 18.8371 | 180232026 |
| String | BTreeMap  | mem_dbg  | 10000000 | 2076110766       | 3.3387  | 182880528 |
| String | BTreeSet  | deepsize | 0        | 24               | 0.0000  | 150       |
| String | BTreeSet  | get-size | 0        | 24               | 0.0000  | 20        |
| String | BTreeSet  | mem_dbg  | 0        | 24               | 0.0000  | 30        |
| String | BTreeSet  | deepsize | 10       | 358              | 12.2549 | 70        |
| String | BTreeSet  | get-size | 10       | 368              | 9.8039  | 60        |
| String | BTreeSet  | mem_dbg  | 10       | 369              | 9.5588  | 60        |
| String | BTreeSet  | deepsize | 100      | 7609             | 26.5328 | 210       |
| String | BTreeSet  | get-size | 100      | 7709             | 25.5672 | 181       |
| String | BTreeSet  | mem_dbg  | 100      | 9952             | 3.9104  | 210       |
| String | BTreeSet  | deepsize | 1000     | 76801            | 22.3738 | 1272      |
| String | BTreeSet  | get-size | 1000     | 77801            | 21.3631 | 1262      |
| String | BTreeSet  | mem_dbg  | 1000     | 102052           | 3.1485  | 1222      |
| String | BTreeSet  | deepsize | 10000    | 777361           | 22.1936 | 12469     |
| String | BTreeSet  | get-size | 10000    | 787361           | 21.1927 | 11788     |
| String | BTreeSet  | mem_dbg  | 10000    | 1032052          | 3.2985  | 35063     |
| String | BTreeSet  | deepsize | 100000   | 7870261          | 21.9754 | 1193348   |
| String | BTreeSet  | get-size | 100000   | 7970261          | 20.9840 | 321107    |
| String | BTreeSet  | mem_dbg  | 100000   | 10422052         | 3.3226  | 1402014   |
| String | BTreeSet  | deepsize | 1000000  | 79681261         | 21.7699 | 14068773  |
| String | BTreeSet  | get-size | 1000000  | 80681261         | 20.7881 | 13674347  |
| String | BTreeSet  | mem_dbg  | 1000000  | 105222052        | 3.3057  | 14691738  |
| String | BTreeSet  | deepsize | 10000000 | 806701261        | 21.5600 | 165190459 |
| String | BTreeSet  | get-size | 10000000 | 816701261        | 20.5876 | 165881355 |
| String | BTreeSet  | mem_dbg  | 10000000 | 1062222052       | 3.2857  | 167570975 |
| String | HashMap   | deepsize | 0        | 48               | 0.0000  | 141       |
| String | HashMap   | get-size | 0        | 48               | 0.0000  | 20        |
| String | HashMap   | mem_dbg  | 0        | 48               | 0.0000  | 120       |
| String | HashMap   | deepsize | 10       | 889              | 12.5860 | 120       |
| String | HashMap   | get-size | 10       | 889              | 12.5860 | 70        |
| String | HashMap   | mem_dbg  | 10       | 978              | 3.8348  | 50        |
| String | HashMap   | deepsize | 100      | 15949            | 5.4089  | 440       |
| String | HashMap   | get-size | 100      | 15949            | 5.4089  | 401       |
| String | HashMap   | mem_dbg  | 100      | 16816            | 0.2669  | 300       |
| String | HashMap   | deepsize | 1000     | 193231           | 6.9139  | 1092      |
| String | HashMap   | get-size | 1000     | 193231           | 6.9139  | 1131      |
| String | HashMap   | mem_dbg  | 1000     | 207196           | 0.1864  | 1092      |
| String | HashMap   | deepsize | 10000    | 1779403          | 6.0558  | 10687     |
| String | HashMap   | get-size | 10000    | 1779403          | 6.0558  | 11106     |
| String | HashMap   | mem_dbg  | 10000    | 1890660          | 0.1820  | 19810     |
| String | HashMap   | deepsize | 100000   | 16614199         | 5.2335  | 80312     |
| String | HashMap   | get-size | 100000   | 16614199         | 5.2335  | 92590     |
| String | HashMap   | mem_dbg  | 100000   | 17500372         | 0.1788  | 115405    |
| String | HashMap   | deepsize | 1000000  | 201150559        | 6.8017  | 4158911   |
| String | HashMap   | get-size | 1000000  | 201150559        | 6.8017  | 4288397   |
| String | HashMap   | mem_dbg  | 1000000  | 215538292        | 0.1355  | 4185060   |
| String | HashMap   | deepsize | 10000000 | 1855233247       | 5.9534  | 24367886  |
| String | HashMap   | get-size | 10000000 | 1855233247       | 5.9534  | 25324196  |
| String | HashMap   | mem_dbg  | 10000000 | 1969861428       | 0.1426  | 23731042  |
| String | HashSet   | deepsize | 0        | 48               | 0.0000  | 271       |
| String | HashSet   | get-size | 0        | 48               | 0.0000  | 20        |
| String | HashSet   | mem_dbg  | 0        | 48               | 0.0000  | 120       |
| String | HashSet   | deepsize | 10       | 488              | 14.0845 | 40        |
| String | HashSet   | get-size | 10       | 488              | 14.0845 | 60        |
| String | HashSet   | mem_dbg  | 10       | 529              | 6.8662  | 40        |
| String | HashSet   | deepsize | 100      | 8021             | 6.1762  | 121       |
| String | HashSet   | get-size | 100      | 8021             | 6.1762  | 120       |
| String | HashSet   | mem_dbg  | 100      | 8504             | 0.5264  | 110       |
| String | HashSet   | deepsize | 1000     | 96833            | 7.8141  | 1002      |
| String | HashSet   | get-size | 1000     | 96833            | 7.8141  | 1081      |
| String | HashSet   | mem_dbg  | 1000     | 104654           | 0.3684  | 1012      |
| String | HashSet   | deepsize | 10000    | 891449           | 6.8497  | 8202      |
| String | HashSet   | get-size | 10000    | 891449           | 6.8497  | 9545      |
| String | HashSet   | mem_dbg  | 10000    | 953554           | 0.3602  | 8012      |
| String | HashSet   | deepsize | 100000   | 8322797          | 5.9263  | 69515     |
| String | HashSet   | get-size | 100000   | 8322797          | 5.9263  | 80692     |
| String | HashSet   | mem_dbg  | 100000   | 8815754          | 0.3543  | 69455     |
| String | HashSet   | deepsize | 1000000  | 100721477        | 7.6882  | 1124243   |
| String | HashSet   | get-size | 1000000  | 100721477        | 7.6882  | 1206018   |
| String | HashSet   | mem_dbg  | 1000000  | 108817754        | 0.2679  | 1153658   |
| String | HashSet   | deepsize | 10000000 | 929022821        | 6.7369  | 9332459   |
| String | HashSet   | get-size | 10000000 | 929022821        | 6.7369  | 10292182  |
| String | HashSet   | mem_dbg  | 10000000 | 993319354        | 0.2823  | 8969769   |
| usize  | BTreeMap  | deepsize | 0        | 24               | 0.0000  | 711       |
| usize  | BTreeMap  | get-size | 0        | 24               | 0.0000  | 30        |
| usize  | BTreeMap  | mem_dbg  | 0        | 24               | 0.0000  | 80        |
| usize  | BTreeMap  | deepsize | 10       | 204              | 5.5556  | 81        |
| usize  | BTreeMap  | get-size | 10       | 184              | 14.8148 | 70        |
| usize  | BTreeMap  | mem_dbg  | 10       | 216              | 0.0000  | 20        |
| usize  | BTreeMap  | deepsize | 100      | 1824             | 48.9933 | 210       |
| usize  | BTreeMap  | get-size | 100      | 1624             | 54.5861 | 210       |
| usize  | BTreeMap  | mem_dbg  | 100      | 3304             | 7.6063  | 30        |
| usize  | BTreeMap  | deepsize | 1000     | 18024            | 47.5925 | 911       |
| usize  | BTreeMap  | get-size | 1000     | 16024            | 53.4078 | 1002      |
| usize  | BTreeMap  | mem_dbg  | 1000     | 34054            | 0.9828  | 10        |
| usize  | BTreeMap  | deepsize | 10000    | 180024           | 47.4757 | 8533      |
| usize  | BTreeMap  | get-size | 10000    | 160024           | 53.3109 | 9044      |
| usize  | BTreeMap  | mem_dbg  | 10000    | 341554           | 0.3472  | 20        |
| usize  | BTreeMap  | deepsize | 100000   | 1800024          | 47.4963 | 110428    |
| usize  | BTreeMap  | get-size | 100000   | 1600024          | 53.3300 | 108494    |
| usize  | BTreeMap  | mem_dbg  | 100000   | 3416554          | 0.3448  | 20        |
| usize  | BTreeMap  | deepsize | 1000000  | 18000024         | 47.4988 | 5077443   |
| usize  | BTreeMap  | get-size | 1000000  | 16000024         | 53.3323 | 4288227   |
| usize  | BTreeMap  | mem_dbg  | 1000000  | 34166554         | 0.3454  | 40        |
| usize  | BTreeMap  | deepsize | 10000000 | 180000024        | 47.4999 | 45992007  |
| usize  | BTreeMap  | get-size | 10000000 | 160000024        | 53.3332 | 44697806  |
| usize  | BTreeMap  | mem_dbg  | 10000000 | 341666554        | 0.3471  | 20        |
| usize  | BTreeSet  | deepsize | 0        | 24               | 0.0000  | 260       |
| usize  | BTreeSet  | get-size | 0        | 24               | 0.0000  | 30        |
| usize  | BTreeSet  | mem_dbg  | 0        | 24               | 0.0000  | 30        |
| usize  | BTreeSet  | deepsize | 10       | 154              | 20.3125 | 70        |
| usize  | BTreeSet  | get-size | 10       | 104              | 18.7500 | 60        |
| usize  | BTreeSet  | mem_dbg  | 10       | 128              | 0.0000  | 30        |
| usize  | BTreeSet  | deepsize | 100      | 1324             | 36.3462 | 220       |
| usize  | BTreeSet  | get-size | 100      | 824              | 60.3846 | 240       |
| usize  | BTreeSet  | mem_dbg  | 100      | 1896             | 8.8462  | 30        |
| usize  | BTreeSet  | deepsize | 1000     | 13024            | 33.8749 | 791       |
| usize  | BTreeSet  | get-size | 1000     | 8024             | 59.2608 | 962       |
| usize  | BTreeSet  | mem_dbg  | 1000     | 19446            | 1.2693  | 10        |
| usize  | BTreeSet  | deepsize | 10000    | 130024           | 33.7072 | 7191      |
| usize  | BTreeSet  | get-size | 10000    | 80024            | 59.1997 | 9965      |
| usize  | BTreeSet  | mem_dbg  | 10000    | 194946           | 0.6067  | 20        |
| usize  | BTreeSet  | deepsize | 100000   | 1300024          | 33.7320 | 169677    |
| usize  | BTreeSet  | get-size | 100000   | 800024           | 59.2192 | 150708    |
| usize  | BTreeSet  | mem_dbg  | 100000   | 1949946          | 0.6026  | 20        |
| usize  | BTreeSet  | deepsize | 1000000  | 13000024         | 33.7361 | 1839226   |
| usize  | BTreeSet  | get-size | 1000000  | 8000024          | 59.2221 | 1651272   |
| usize  | BTreeSet  | mem_dbg  | 1000000  | 19499946         | 0.6046  | 20        |
| usize  | BTreeSet  | deepsize | 10000000 | 130000024        | 33.7377 | 26975106  |
| usize  | BTreeSet  | get-size | 10000000 | 80000024         | 59.2232 | 26713941  |
| usize  | BTreeSet  | mem_dbg  | 10000000 | 194999946        | 0.6066  | 30        |
| usize  | HashMap   | deepsize | 0        | 48               | 0.0000  | 340       |
| usize  | HashMap   | get-size | 0        | 48               | 0.0000  | 20        |
| usize  | HashMap   | mem_dbg  | 0        | 48               | 0.0000  | 121       |
| usize  | HashMap   | deepsize | 10       | 272              | 19.0476 | 40        |
| usize  | HashMap   | get-size | 10       | 272              | 19.0476 | 30        |
| usize  | HashMap   | mem_dbg  | 10       | 336              | 0.0000  | 20        |
| usize  | HashMap   | deepsize | 100      | 1840             | 17.8571 | 101       |
| usize  | HashMap   | get-size | 100      | 1840             | 17.8571 | 20        |
| usize  | HashMap   | mem_dbg  | 100      | 2240             | 0.0000  | 10        |
| usize  | HashMap   | deepsize | 1000     | 28720            | 17.6606 | 982       |
| usize  | HashMap   | get-size | 1000     | 28720            | 17.6606 | 10        |
| usize  | HashMap   | mem_dbg  | 1000     | 34880            | 0.0000  | 20        |
| usize  | HashMap   | deepsize | 10000    | 229424           | 17.6487 | 8182      |
| usize  | HashMap   | get-size | 10000    | 229424           | 17.6487 | 20        |
| usize  | HashMap   | mem_dbg  | 10000    | 278592           | 0.0000  | 10        |
| usize  | HashMap   | deepsize | 100000   | 1835056          | 17.6473 | 61744     |
| usize  | HashMap   | get-size | 100000   | 1835056          | 17.6473 | 20        |
| usize  | HashMap   | mem_dbg  | 100000   | 2228288          | 0.0000  | 20        |
| usize  | HashMap   | deepsize | 1000000  | 29360176         | 17.6471 | 959925    |
| usize  | HashMap   | get-size | 1000000  | 29360176         | 17.6471 | 30        |
| usize  | HashMap   | mem_dbg  | 1000000  | 35651648         | 0.0000  | 20        |
| usize  | HashMap   | deepsize | 10000000 | 234881072        | 17.6471 | 8068203   |
| usize  | HashMap   | get-size | 10000000 | 234881072        | 17.6471 | 30        |
| usize  | HashMap   | mem_dbg  | 10000000 | 285212736        | 0.0000  | 40        |
| usize  | HashSet   | deepsize | 0        | 48               | 0.0000  | 130       |
| usize  | HashSet   | get-size | 0        | 48               | 0.0000  | 20        |
| usize  | HashSet   | mem_dbg  | 0        | 48               | 0.0000  | 40        |
| usize  | HashSet   | deepsize | 10       | 160              | 23.0769 | 40        |
| usize  | HashSet   | get-size | 10       | 160              | 23.0769 | 20        |
| usize  | HashSet   | mem_dbg  | 10       | 208              | 0.0000  | 20        |
| usize  | HashSet   | deepsize | 100      | 944              | 22.3684 | 120       |
| usize  | HashSet   | get-size | 100      | 944              | 22.3684 | 20        |
| usize  | HashSet   | mem_dbg  | 100      | 1216             | 0.0000  | 20        |
| usize  | HashSet   | deepsize | 1000     | 14384            | 22.2318 | 1002      |
| usize  | HashSet   | get-size | 1000     | 14384            | 22.2318 | 20        |
| usize  | HashSet   | mem_dbg  | 1000     | 18496            | 0.0000  | 20        |
| usize  | HashSet   | deepsize | 10000    | 114736           | 22.2234 | 8052      |
| usize  | HashSet   | get-size | 10000    | 114736           | 22.2234 | 10        |
| usize  | HashSet   | mem_dbg  | 10000    | 147520           | 0.0000  | 20        |
| usize  | HashSet   | deepsize | 100000   | 917552           | 22.2224 | 63266     |
| usize  | HashSet   | get-size | 100000   | 917552           | 22.2224 | 20        |
| usize  | HashSet   | mem_dbg  | 100000   | 1179712          | 0.0000  | 20        |
| usize  | HashSet   | deepsize | 1000000  | 14680112         | 22.2222 | 962287    |
| usize  | HashSet   | get-size | 1000000  | 14680112         | 22.2222 | 30        |
| usize  | HashSet   | mem_dbg  | 1000000  | 18874432         | 0.0000  | 20        |
| usize  | HashSet   | deepsize | 10000000 | 117440560        | 22.2222 | 8072369   |
| usize  | HashSet   | get-size | 10000000 | 117440560        | 22.2222 | 40        |
| usize  | HashSet   | mem_dbg  | 10000000 | 150995008        | 0.0000  | 40        |

## Aggregated Results

| Type   | Container | Crate    | Error (%)       | Time/Elem (ns)  |
|--------|-----------|----------|-----------------|-----------------|
| String | BTreeMap  | deepsize | 21.53 ± 8.88    | 7.15 ± 6.81     |
|        |           | get-size | 16.04 ± 7.74    | **6.62 ± 6.89** |
|        |           | mem_dbg  | **3.17 ± 1.45** | 7.75 ± 7.36     |
|        | BTreeSet  | deepsize | 18.58 ± 8.51    | 7.73 ± 6.47     |
|        |           | get-size | 17.54 ± 8.38    | **6.25 ± 6.35** |
|        |           | mem_dbg  | **3.73 ± 2.65** | 8.33 ± 6.61     |
|        | HashMap   | deepsize | 6.12 ± 3.42     | 3.71 ± 3.94     |
|        |           | get-size | 6.12 ± 3.42     | 3.00 ± 2.25     |
|        |           | mem_dbg  | **0.62 ± 1.30** | **2.68 ± 1.48** |
|        | HashSet   | deepsize | 6.91 ± 3.82     | 1.40 ± 1.16     |
|        |           | get-size | 6.91 ± 3.82     | 1.75 ± 1.88     |
|        |           | mem_dbg  | **1.13 ± 2.32** | **1.38 ± 1.17** |
| usize  | BTreeMap  | deepsize | 36.51 ± 20.88   | 3.25 ± 2.76     |
|        |           | get-size | 42.01 ± 21.73   | 2.98 ± 2.33     |
|        |           | mem_dbg  | **1.25 ± 2.59** | **0.33 ± 0.74** |
|        | BTreeSet  | deepsize | 28.18 ± 12.41   | 2.42 ± 2.14     |
|        |           | get-size | 46.91 ± 23.70   | 2.31 ± 1.75     |
|        |           | mem_dbg  | **1.57 ± 2.97** | **0.47 ± 1.12** |
|        | HashMap   | deepsize | 15.64 ± 6.34    | 1.31 ± 1.19     |
|        |           | get-size | 15.64 ± 6.34    | 0.46 ± 1.12     |
|        |           | mem_dbg  | **0.00 ± 0.00** | **0.30 ± 0.75** |
|        | HashSet   | deepsize | 19.57 ± 7.91    | 1.34 ± 1.18     |
|        |           | get-size | 19.57 ± 7.91    | **0.32 ± 0.75** |
|        |           | mem_dbg  | **0.00 ± 0.00** | 0.32 ± 0.75     |
