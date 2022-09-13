//
// Copyright 2017 Christian Reitwiessner
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
// 2019 OKIMS
//      ported to solidity 0.6
//      fixed linter warnings
//      added requiere error messages
//
//
// SPDX-License-Identifier: GPL-3.0
pragma solidity 0.8.9;
import "hardhat/console.sol";
library Pairing {
    struct G1Point {
        uint X;
        uint Y;
    }
    // Encoding of field elements is: X[0] * z + X[1]
    struct G2Point {
        uint[2] X;
        uint[2] Y;
    }
    /// @return the generator of G1
    function P1() internal pure returns (G1Point memory) {
        return G1Point(1, 2); //gas-reporter #28
    }
    /// @return the generator of G2
    function P2() internal pure returns (G2Point memory) {
        // Original code point //gas-reporter #32
        return G2Point(
            [11559732032986387107991004021392285783925812861821192530917403151452391805634,
             10857046999023057135944570762232829481370756359578518086990519993285655852781],
            [4082367875863433681332203403145435568316851327593401208105741076214120093531,
             8495653923123431417604973247489272438418190587263600148770280649306958101930]
        ); //gas-reporter #38

/*
        // Changed by Jordi point
        return G2Point(
            [10857046999023057135944570762232829481370756359578518086990519993285655852781,
             11559732032986387107991004021392285783925812861821192530917403151452391805634],
            [8495653923123431417604973247489272438418190587263600148770280649306958101930,
             4082367875863433681332203403145435568316851327593401208105741076214120093531]
        ); //gas-reporter #47
*/
    }
    /// @return r the negation of p, i.e. p.addition(p.negate()) should be zero.
    function negate(G1Point memory p) internal pure returns (G1Point memory r) {
        // The prime q in the base field F_q for G1 //gas-reporter #52
        uint q = 21888242871839275222246405745257275088696311157297823662689037894645226208583; //gas-reporter #53
        if (p.X == 0 && p.Y == 0)
            return G1Point(0, 0);
        return G1Point(p.X, q - (p.Y % q));
    }
    /// @return r the sum of two points of G1
    function addition(G1Point memory p1, G1Point memory p2) internal view returns (G1Point memory r) {
        uint[4] memory input; //gas-reporter #60
        input[0] = p1.X; //gas-reporter #61
        input[1] = p1.Y; //gas-reporter #62
        input[2] = p2.X; //gas-reporter #63
        input[3] = p2.Y; //gas-reporter #64
        bool success; //gas-reporter #65
        // solium-disable-next-line security/no-inline-assembly //gas-reporter #66
        assembly {
            success := staticcall(sub(gas(), 2000), 6, input, 0xc0, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success,"pairing-add-failed"); //gas-reporter #72
    }
    /// @return r the product of a point on G1 and a scalar, i.e.
    /// p == p.scalar_mul(1) and p.addition(p) == p.scalar_mul(2) for all points p.
    function scalar_mul(G1Point memory p, uint s) internal view returns (G1Point memory r) {
        uint[3] memory input; //gas-reporter #77
        input[0] = p.X; //gas-reporter #78
        input[1] = p.Y; //gas-reporter #79
        input[2] = s; //gas-reporter #80
        bool success; //gas-reporter #81
        // solium-disable-next-line security/no-inline-assembly //gas-reporter #82
        assembly {
            success := staticcall(sub(gas(), 2000), 7, input, 0x80, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require (success,"pairing-mul-failed"); //gas-reporter #88
    }
    /// @return the result of computing the pairing check
    /// e(p1[0], p2[0]) *  .... * e(p1[n], p2[n]) == 1
    /// For example pairing([P1(), P1().negate()], [P2(), P2()]) should
    /// return true.
    function pairing(G1Point[] memory p1, G2Point[] memory p2) internal view returns (bool) {
        require(p1.length == p2.length,"pairing-lengths-failed"); //gas-reporter #95
        uint elements = p1.length; //gas-reporter #96
        uint inputSize = elements * 6; //gas-reporter #97
        uint[] memory input = new uint[](inputSize); //gas-reporter #98
        for (uint i = 0; i < elements; i++)
        {
            input[i * 6 + 0] = p1[i].X; //gas-reporter #101
            input[i * 6 + 1] = p1[i].Y; //gas-reporter #102
            input[i * 6 + 2] = p2[i].X[0]; //gas-reporter #103
            input[i * 6 + 3] = p2[i].X[1]; //gas-reporter #104
            input[i * 6 + 4] = p2[i].Y[0]; //gas-reporter #105
            input[i * 6 + 5] = p2[i].Y[1]; //gas-reporter #106
        }
        uint[1] memory out; //gas-reporter #108
        bool success; //gas-reporter #109
        // solium-disable-next-line security/no-inline-assembly //gas-reporter #110
        assembly {
            success := staticcall(sub(gas(), 2000), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success,"pairing-opcode-failed"); //gas-reporter #116
        return out[0] != 0;
    }
    /// Convenience method for a pairing check for two pairs.
    function pairingProd2(G1Point memory a1, G2Point memory a2, G1Point memory b1, G2Point memory b2) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](2); //gas-reporter #121
        G2Point[] memory p2 = new G2Point[](2); //gas-reporter #122
        p1[0] = a1; //gas-reporter #123
        p1[1] = b1; //gas-reporter #124
        p2[0] = a2; //gas-reporter #125
        p2[1] = b2; //gas-reporter #126
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for three pairs.
    function pairingProd3(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](3); //gas-reporter #135
        G2Point[] memory p2 = new G2Point[](3); //gas-reporter #136
        p1[0] = a1; //gas-reporter #137
        p1[1] = b1; //gas-reporter #138
        p1[2] = c1; //gas-reporter #139
        p2[0] = a2; //gas-reporter #140
        p2[1] = b2; //gas-reporter #141
        p2[2] = c2; //gas-reporter #142
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for four pairs.
    function pairingProd4(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2,
            G1Point memory d1, G2Point memory d2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](4); //gas-reporter #152
        G2Point[] memory p2 = new G2Point[](4); //gas-reporter #153
        p1[0] = a1; //gas-reporter #154
        p1[1] = b1; //gas-reporter #155
        p1[2] = c1; //gas-reporter #156
        p1[3] = d1; //gas-reporter #157
        p2[0] = a2; //gas-reporter #158
        p2[1] = b2; //gas-reporter #159
        p2[2] = c2; //gas-reporter #160
        p2[3] = d2; //gas-reporter #161
        return pairing(p1, p2);
    }
}
contract Verifier {
    using Pairing for *; //gas-reporter #166
    struct VerifyingKey {
        Pairing.G1Point alfa1; //gas-reporter #168
        Pairing.G2Point beta2; //gas-reporter #169
        Pairing.G2Point gamma2; //gas-reporter #170
        Pairing.G2Point delta2; //gas-reporter #171
        Pairing.G1Point[] IC; //gas-reporter #172
    }
    struct Proof {
        Pairing.G1Point A; //gas-reporter #175
        Pairing.G2Point B; //gas-reporter #176
        Pairing.G1Point C; //gas-reporter #177
    }
    function verifyingKey() internal pure returns (VerifyingKey memory vk) {
        vk.alfa1 = Pairing.G1Point( //gas-reporter #180
            20491192805390485299153009773594534940189261866228447918068658471970481763042,
            9383485363053290200918347156157836566562967994039712273449902621266178545958
        ); //gas-reporter #183

        vk.beta2 = Pairing.G2Point(
            [4252822878758300859123897981450591353533073413197771768651442665752259397132,
             6375614351688725206403948262868962793625744043794305715222011528459656738731],
            [21847035105528745403288232691147584728191162732299865338377159692350059136679,
             10505242626370262277552901082094356697409835680220590971873171140371331206856]
        ); //gas-reporter #190
        vk.gamma2 = Pairing.G2Point(
            [11559732032986387107991004021392285783925812861821192530917403151452391805634,
             10857046999023057135944570762232829481370756359578518086990519993285655852781],
            [4082367875863433681332203403145435568316851327593401208105741076214120093531,
             8495653923123431417604973247489272438418190587263600148770280649306958101930]
        ); //gas-reporter #196
        vk.delta2 = Pairing.G2Point(
            [11559732032986387107991004021392285783925812861821192530917403151452391805634,
             10857046999023057135944570762232829481370756359578518086990519993285655852781],
            [4082367875863433681332203403145435568316851327593401208105741076214120093531,
             8495653923123431417604973247489272438418190587263600148770280649306958101930]
        ); //gas-reporter #202
        vk.IC = new Pairing.G1Point[](62); //gas-reporter #203

        vk.IC[0] = Pairing.G1Point(
            12869512517627091097920922043935045234313294476049454392258627692903114931618,
            2939107530455968239376203776089805992716396643981078871158057619954123207321
        ); //gas-reporter #208

        vk.IC[1] = Pairing.G1Point(
            2552919366921949944963947340529375206710240572054362523789594527939562216030,
            6684917839250138000321977699205053723944390261596783788127383703442128194163
        ); //gas-reporter #213

        vk.IC[2] = Pairing.G1Point(
            2832703435341806433357471471963789180728635060151169163192410023443863381612,
            11614215983686905628757230380535793241810426992218542120857818996787491470808
        ); //gas-reporter #218

        vk.IC[3] = Pairing.G1Point(
            5660810780620426081419420406009764106663806723637609604413726353295512143364,
            1524500384161352995682805178166612344652647752550982213315848489635844202163
        ); //gas-reporter #223

        vk.IC[4] = Pairing.G1Point(
            14566085506037067950165691192677604356427680496909597518321399882970401076504,
            11097848988624681992719900126074417360228699445829169676989636884392330495110
        ); //gas-reporter #228

        vk.IC[5] = Pairing.G1Point(
            13897362113538253166035892877719588132237271272744513032901431161157704651434,
            7427717736760655727612748264541623371637221258064337917940195893861802897706
        ); //gas-reporter #233

        vk.IC[6] = Pairing.G1Point(
            13439198523807493022234500946119009390743073331725729209626199290289371437924,
            2237599212214401530249043579308896830779190106606432562781455085215390822636
        ); //gas-reporter #238

        vk.IC[7] = Pairing.G1Point(
            21065080018431323832427333701890240692440103039258117848170184337219104142973,
            11647866046010090115048014299177911217049322865665676046656377185111223397852
        ); //gas-reporter #243

        vk.IC[8] = Pairing.G1Point(
            19052445088719402350844264856352929863877810844535051895239792881991653354996,
            5093761892518863864282255703460180010595362708980960011382358664629198012172
        ); //gas-reporter #248

        vk.IC[9] = Pairing.G1Point(
            633875774265728708140638267637119892951619009108780754797521452270932126502,
            6775270345624513012531614055582427701053990608664808231229173146628523452244
        ); //gas-reporter #253

        vk.IC[10] = Pairing.G1Point(
            9427064001809808197909495413742796204481571512574401740072395459850674122851,
            10590971200058992055746496651364551316061293429439302372427882195510550939059
        ); //gas-reporter #258

        vk.IC[11] = Pairing.G1Point(
            6410776721702470007474297201650041473965556535256157667415179432628450462060,
            19793266480451427679057283347721891526343242458584820627736297275862473824403
        ); //gas-reporter #263

        vk.IC[12] = Pairing.G1Point(
            9671263810883737373580405938500773815053140770083587422575906353799835107010,
            18442988956627257706347647229641144183683924577180339466985942316515558294108
        ); //gas-reporter #268

        vk.IC[13] = Pairing.G1Point(
            19309132673109480846393982551990991572689446112364015695400483083242509458100,
            4810215899696713657022101125208088402465166797061400516589497887883529422891
        ); //gas-reporter #273

        vk.IC[14] = Pairing.G1Point(
            18913670191220907406053276050524703236995492491554930195553603121819070338112,
            10938108712613412719229528838669417500778405374921972660342735581475858986072
        ); //gas-reporter #278

        vk.IC[15] = Pairing.G1Point(
            5333955185230100626471194216336040256148031903710227020575360175269092076629,
            11811775014790350420931646776109673237680654297119807800500025378885698210891
        ); //gas-reporter #283

        vk.IC[16] = Pairing.G1Point(
            2946849571314615265974969845476649817962682431314772251217885118384610551071,
            17588206853706887781586510808389898757183165624088382268919404302349640462103
        ); //gas-reporter #288

        vk.IC[17] = Pairing.G1Point(
            111531782171600190781207737358520523925449657531481282432050120599349993608,
            19401625178713563677682280673439929588520318988250949349536280915102695882734
        ); //gas-reporter #293

        vk.IC[18] = Pairing.G1Point(
            21209913452415969902981739214573086623529416579638428122850115734816879429306,
            1403011171983556216103249939948285289442535648026839989539996724937363331298
        ); //gas-reporter #298

        vk.IC[19] = Pairing.G1Point(
            151279437642567280108998035474920673466648235894026262175054180285221453437,
            5945876360380288336502200466671282768498521492670389666181786750135454202854
        ); //gas-reporter #303

        vk.IC[20] = Pairing.G1Point(
            11018880120743343845498149436739126814331094791197533639295435980577311708473,
            8821514552698527422615041233156060692373476380745890536532784075014993465818
        ); //gas-reporter #308

        vk.IC[21] = Pairing.G1Point(
            18372799702296426633413291673173850957107616052259024813160518933763200006714,
            13197019660474273777643089371481981253650884486581033910302549898840794461205
        ); //gas-reporter #313

        vk.IC[22] = Pairing.G1Point(
            6788078929270653511362471607531388144398947020955700143853270261192776155916,
            20825819489751008846645075421050888570674062510607761461010798746604779185030
        ); //gas-reporter #318

        vk.IC[23] = Pairing.G1Point(
            17758592847113057178507402055771594438426186639445812966896007053776577064981,
            20824043695663014293336986206505782606133004425560899614794582645408371308061
        ); //gas-reporter #323

        vk.IC[24] = Pairing.G1Point(
            5317908277774730507701020595712675320174855276957405763626591758191925099522,
            2492526684249181415144153076542403207202546925954106874852092116436659509194
        ); //gas-reporter #328

        vk.IC[25] = Pairing.G1Point(
            6532336994333345046196991289142844349310182836758497964260486522118834217479,
            18072037546387628480339166712615280238417904572644410438040059214394593167056
        ); //gas-reporter #333

        vk.IC[26] = Pairing.G1Point(
            15399188416234590228784477432620765253601509682490581667857501939428215897126,
            12574654705117002892097784395250559079124675850302635333742721080354154733645
        ); //gas-reporter #338

        vk.IC[27] = Pairing.G1Point(
            8749907726535450448268732553291394586089129838019689499329326294617435218987,
            3320017498608698212254539640169744942402169253994334530335139721362498141632
        ); //gas-reporter #343

        vk.IC[28] = Pairing.G1Point(
            15499272035610637053670025794133697031415835963719137037151765934809698793250,
            19250475978916482181067201944709206901086821863437787805289738837732643949884
        ); //gas-reporter #348

        vk.IC[29] = Pairing.G1Point(
            21170407386567429137036824397584018694742249256484646283753080429427570696330,
            9375985901371488593913045076646213361067687403277126293958758640149278108942
        ); //gas-reporter #353

        vk.IC[30] = Pairing.G1Point(
            9369680763590720722834435998591765380150991959989560116620359398871377223490,
            13585549471491026209731820591684973423296623693239449859634562502877415801553
        ); //gas-reporter #358

        vk.IC[31] = Pairing.G1Point(
            2908334395793836225045958113742054466331452695043093503366205880521645835987,
            12877330490940519784785108784356432009602672434370368047290878320468906392671
        ); //gas-reporter #363

        vk.IC[32] = Pairing.G1Point(
            19160623672929572182899087708288133358036333062388870790517949803332365611145,
            13659733740708127838160229824811149946659874353089982642943751163409964488122
        ); //gas-reporter #368

        vk.IC[33] = Pairing.G1Point(
            19484576847090271677798770989691206067522122585977181363302721855254635492345,
            6640213481599874564841956233816929700736604835058033568842811460612278487600
        ); //gas-reporter #373

        vk.IC[34] = Pairing.G1Point(
            20219744880112120854766314299721381273036260853971895082262512062777344326011,
            10916393208131378945751212112273310078298156938483379702160532310753419952441
        ); //gas-reporter #378

        vk.IC[35] = Pairing.G1Point(
            15545216056357690682894762791649783182132329844870546667428844446811481530783,
            2289920225811642004677181272493469503216121844180602781554882870403322811133
        ); //gas-reporter #383

        vk.IC[36] = Pairing.G1Point(
            16475088248407686040369636009460360806823646045285245234518472472835917867600,
            10817473200546048459117005958849844408958140394576242746019258200700712376941
        ); //gas-reporter #388

        vk.IC[37] = Pairing.G1Point(
            5757388973909399226194015342355685105428504777747440849423916029104254990007,
            10984067296864730094100368724944807490694082485568453943534642416234468036093
        ); //gas-reporter #393

        vk.IC[38] = Pairing.G1Point(
            4437129276331010402692903543858995426803060024181897985980152408357453255324,
            21727187413548512387553335082396055176152049047950287225726401196864251750284
        ); //gas-reporter #398

        vk.IC[39] = Pairing.G1Point(
            4395829192726328918063828989617627086289953279915015198449758331616614966562,
            10934146896556205458708834956683702426761389095309240514648366520587534230224
        ); //gas-reporter #403

        vk.IC[40] = Pairing.G1Point(
            5801749955551881144119955489694912965788100089904991864103700740732164117583,
            3732212209149184587392371259840821714932701131484027813787126880569729317023
        ); //gas-reporter #408

        vk.IC[41] = Pairing.G1Point(
            21805367755311740395219269670743529707112970645045274314497883401078805829455,
            8872176804371089297014841368768872546779778130310413798251414537969859709424
        ); //gas-reporter #413

        vk.IC[42] = Pairing.G1Point(
            12587700621593042014388946193157079668047613132900562182761602943067387719003,
            14148096846336754589644545815382697216808838305901792447635855035821606286745
        ); //gas-reporter #418

        vk.IC[43] = Pairing.G1Point(
            85752041269572968057861624758734830994187147240458703500987103914722665153,
            1147821778591696040277691557981041268017891570710904339356047701194551536902
        ); //gas-reporter #423

        vk.IC[44] = Pairing.G1Point(
            18697382907102364810593743456407739883093108944034733774649428113978930450316,
            16782353252700873399337708857168890675030552038480359447864343270492333505080
        ); //gas-reporter #428

        vk.IC[45] = Pairing.G1Point(
            3724143042446267070351286548737460616894744407860871290351958521399084370151,
            9147485607067015783373825872065251694344663953573064645704068637479773093533
        ); //gas-reporter #433

        vk.IC[46] = Pairing.G1Point(
            11384168397622812710576606753015940777029290296091149614025005902325926018008,
            6894645710450710620318510492578759562828200327667435956130858824361635163354
        ); //gas-reporter #438

        vk.IC[47] = Pairing.G1Point(
            15040606123702443326794684657658100547200461002643715798089542702808237398571,
            16569836928108407810855767776487911662848603787742233674481236828982902983419
        ); //gas-reporter #443

        vk.IC[48] = Pairing.G1Point(
            6472459022897642762982865534132215849103810624281482042729891703694901465466,
            1496036388029544309245755953758059586762985296327148239646864879017644535630
        ); //gas-reporter #448

        vk.IC[49] = Pairing.G1Point(
            10448408235091215742854518055490120456594992680657729017922926923475137903851,
            19552592528340652142204513626290379334844774410881117136567500034397191393094
        ); //gas-reporter #453

        vk.IC[50] = Pairing.G1Point(
            18630881280008457760038722430250204674972121661094889201596347526408063499972,
            21675334452054924198675128789945151742430800772251855306715048073229434539862
        ); //gas-reporter #458

        vk.IC[51] = Pairing.G1Point(
            2266675084033468132613217068180901782455133622651535546537199505604965957724,
            17520668955369640506997237312319864856327877006866230433034192454030608660053
        ); //gas-reporter #463

        vk.IC[52] = Pairing.G1Point(
            7418820165404057241439340311720501262658409150696621479959523151246150094828,
            14869486437995228973703587065855147211153863331139918162446632772521319696935
        ); //gas-reporter #468

        vk.IC[53] = Pairing.G1Point(
            16389290774548024851031408137788811475473958773704696945683519617627007458290,
            20629944008034644271044259772804213806778035041859735066885950674782929244674
        ); //gas-reporter #473

        vk.IC[54] = Pairing.G1Point(
            12295562099811620263383829409689107401160293276317391295788052262067171289905,
            14033379777731886510784738991436672027713580073425569830584985846795902159387
        ); //gas-reporter #478

        vk.IC[55] = Pairing.G1Point(
            4770177314333019262828721742934328876111065402092023944101295837899299677635,
            815819263310583079815791776022519901800328702806712096982753433661863634703
        ); //gas-reporter #483

        vk.IC[56] = Pairing.G1Point(
            5312640850080932168410832526509425495098873443824976836022457215294757690731,
            18100457249547885150274644464014219293657716138421573090426195478517013963124
        ); //gas-reporter #488

        vk.IC[57] = Pairing.G1Point(
            8603354702162740949006864328258619174231496314296473198100991227116112982015,
            1258352243111485062725623411977732834345848842418078902650040323413123604931
        ); //gas-reporter #493

        vk.IC[58] = Pairing.G1Point(
            9256941148970272202735102697185696499794895151472532986527462453951089600278,
            3288088055833367292653930541934943152819458406572995827712466050949430248333
        ); //gas-reporter #498

        vk.IC[59] = Pairing.G1Point(
            11399762694264116392817463887787755825469032239821370866656770417520908978269,
            6157479904077993289099144730162316234127890742511863630391536870490643318958
        ); //gas-reporter #503

        vk.IC[60] = Pairing.G1Point(
            14725297658318366270672741408721724588439804446183833615863683812773674219460,
            7968815474780929795438546925080687929270711333059993517739769457897113367857
        ); //gas-reporter #508

        vk.IC[61] = Pairing.G1Point(
            11493203305910715686845891184896786035678844307310767610219836190476442863730,
            19958281415308224773314192643439017305105199431024311073831007560321915719666
        ); //gas-reporter #513

    }
    function verify(uint[] memory input, Proof memory proof) internal view returns (uint) {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617; //gas-reporter #517
        VerifyingKey memory vk = verifyingKey(); //gas-reporter #518
        require(input.length + 1 == vk.IC.length,"verifier-bad-input"); //gas-reporter #519
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0); //gas-reporter #521
        for (uint i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field,"verifier-gte-snark-scalar-field"); //gas-reporter #523
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.IC[i + 1], input[i])); //gas-reporter #524
        }
        vk_x = Pairing.addition(vk_x, vk.IC[0]); //gas-reporter #526
        if (!Pairing.pairingProd4(
            Pairing.negate(proof.A), proof.B,
            vk.alfa1, vk.beta2,
            vk_x, vk.gamma2,
            proof.C, vk.delta2
        )) return 1; //gas-reporter #532
        return 0;
    }
    /// @return r  bool true if proof is valid
    function verifyProof(
            uint[2] memory a,
            uint[2][2] memory b,
            uint[2] memory c,
            uint[61] memory input
        ) public view returns (bool r) {
        Proof memory proof; //gas-reporter #542
        proof.A = Pairing.G1Point(a[0], a[1]); //gas-reporter #543
        proof.B = Pairing.G2Point([b[0][0], b[0][1]], [b[1][0], b[1][1]]); //gas-reporter #544
        proof.C = Pairing.G1Point(c[0], c[1]); //gas-reporter #545
        uint[] memory inputValues = new uint[](input.length); //gas-reporter #546
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i]; //gas-reporter #548
        }
        if (verify(inputValues, proof) == 0) {
            return true;
        } else {
            return false;
        }
    }
}