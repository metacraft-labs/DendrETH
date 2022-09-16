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
        return G1Point(1, 2);
    }
    /// @return the generator of G2
    function P2() internal pure returns (G2Point memory) {
        // Original code point
        return G2Point(
            [11559732032986387107991004021392285783925812861821192530917403151452391805634,
             10857046999023057135944570762232829481370756359578518086990519993285655852781],
            [4082367875863433681332203403145435568316851327593401208105741076214120093531,
             8495653923123431417604973247489272438418190587263600148770280649306958101930]
        );

/*
        // Changed by Jordi point
        return G2Point(
            [10857046999023057135944570762232829481370756359578518086990519993285655852781,
             11559732032986387107991004021392285783925812861821192530917403151452391805634],
            [8495653923123431417604973247489272438418190587263600148770280649306958101930,
             4082367875863433681332203403145435568316851327593401208105741076214120093531]
        );
*/
    }
    /// @return r the negation of p, i.e. p.addition(p.negate()) should be zero.
    function negate(G1Point memory p) internal pure returns (G1Point memory r) {
        // The prime q in the base field F_q for G1
        uint q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
        if (p.X == 0 && p.Y == 0)
            return G1Point(0, 0);
        return G1Point(p.X, q - (p.Y % q));
    }
    /// @return r the sum of two points of G1
    function addition(G1Point memory p1, G1Point memory p2) internal view returns (G1Point memory r) {
        uint[4] memory input;
        input[0] = p1.X;
        input[1] = p1.Y;
        input[2] = p2.X;
        input[3] = p2.Y;
        bool success;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            success := staticcall(sub(gas(), 2000), 6, input, 0xc0, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success,"pairing-add-failed");
    }
    /// @return r the product of a point on G1 and a scalar, i.e.
    /// p == p.scalar_mul(1) and p.addition(p) == p.scalar_mul(2) for all points p.
    function scalar_mul(G1Point memory p, uint s) internal view returns (G1Point memory r) {
        uint[3] memory input;
        input[0] = p.X;
        input[1] = p.Y;
        input[2] = s;
        bool success;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            success := staticcall(sub(gas(), 2000), 7, input, 0x80, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require (success,"pairing-mul-failed");
    }
    /// @return the result of computing the pairing check
    /// e(p1[0], p2[0]) *  .... * e(p1[n], p2[n]) == 1
    /// For example pairing([P1(), P1().negate()], [P2(), P2()]) should
    /// return true.
    function pairing(G1Point[] memory p1, G2Point[] memory p2) internal view returns (bool) {
        require(p1.length == p2.length,"pairing-lengths-failed");
        uint elements = p1.length;
        uint inputSize = elements * 6;
        uint[] memory input = new uint[](inputSize);
        for (uint i = 0; i < elements; i++)
        {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[0];
            input[i * 6 + 3] = p2[i].X[1];
            input[i * 6 + 4] = p2[i].Y[0];
            input[i * 6 + 5] = p2[i].Y[1];
        }
        uint[1] memory out;
        bool success;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            success := staticcall(sub(gas(), 2000), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success,"pairing-opcode-failed");
        return out[0] != 0;
    }
    /// Convenience method for a pairing check for two pairs.
    function pairingProd2(G1Point memory a1, G2Point memory a2, G1Point memory b1, G2Point memory b2) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](2);
        G2Point[] memory p2 = new G2Point[](2);
        p1[0] = a1;
        p1[1] = b1;
        p2[0] = a2;
        p2[1] = b2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for three pairs.
    function pairingProd3(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](3);
        G2Point[] memory p2 = new G2Point[](3);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for four pairs.
    function pairingProd4(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2,
            G1Point memory d1, G2Point memory d2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](4);
        G2Point[] memory p2 = new G2Point[](4);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p1[3] = d1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        p2[3] = d2;
        return pairing(p1, p2);
    }
}
contract Verifier {
    using Pairing for *;
    struct VerifyingKey {
        Pairing.G1Point alfa1;
        Pairing.G2Point beta2;
        Pairing.G2Point gamma2;
        Pairing.G2Point delta2;
        Pairing.G1Point[] IC;
    }
    struct Proof {
        Pairing.G1Point A;
        Pairing.G2Point B;
        Pairing.G1Point C;
    }
    function verifyingKey() internal pure returns (VerifyingKey memory vk) {
        vk.alfa1 = Pairing.G1Point(
            20491192805390485299153009773594534940189261866228447918068658471970481763042,
            9383485363053290200918347156157836566562967994039712273449902621266178545958
        );

        vk.beta2 = Pairing.G2Point(
            [4252822878758300859123897981450591353533073413197771768651442665752259397132,
             6375614351688725206403948262868962793625744043794305715222011528459656738731],
            [21847035105528745403288232691147584728191162732299865338377159692350059136679,
             10505242626370262277552901082094356697409835680220590971873171140371331206856]
        );
        vk.gamma2 = Pairing.G2Point(
            [11559732032986387107991004021392285783925812861821192530917403151452391805634,
             10857046999023057135944570762232829481370756359578518086990519993285655852781],
            [4082367875863433681332203403145435568316851327593401208105741076214120093531,
             8495653923123431417604973247489272438418190587263600148770280649306958101930]
        );
        vk.delta2 = Pairing.G2Point(
            [11559732032986387107991004021392285783925812861821192530917403151452391805634,
             10857046999023057135944570762232829481370756359578518086990519993285655852781],
            [4082367875863433681332203403145435568316851327593401208105741076214120093531,
             8495653923123431417604973247489272438418190587263600148770280649306958101930]
        );
        vk.IC = new Pairing.G1Point[](31);

        vk.IC[0] = Pairing.G1Point(
            9879475712677537408623622991214420375825067082345993445981781411616812804080,
            4373167706371101922902097176278825058784281903136681093158495163462032119723
        );

        vk.IC[1] = Pairing.G1Point(
            13755013584842966412496666910316533563181244332544143391022095864011518190272,
            9332746614031816088407641296883324848795477527770147741654051885301222436595
        );

        vk.IC[2] = Pairing.G1Point(
            17106311849596638116266496801997659514999847961926550773479916194117904982133,
            1399600859481525132225702401852690499982550543426783806967105682777877127745
        );

        vk.IC[3] = Pairing.G1Point(
            19622204296754233436745205194378354161980555061074115178064543163393236645797,
            14038454796201280691443732054062515484029500987132888046656856145016681275187
        );

        vk.IC[4] = Pairing.G1Point(
            9168160612002775113655818319209955083899048291068423819061495799327340127121,
            13314401299004911063090538858331960259095551409523207450006006326543035702781
        );

        vk.IC[5] = Pairing.G1Point(
            18067650115464614931577503626825236751795968862420461313381307969393321083737,
            4157100926580945147215156816661889340507136522995373607876078408252619903
        );

        vk.IC[6] = Pairing.G1Point(
            19180656919399626658838450911962835723211783215314788500936659223502197662769,
            11360509784674188211540593503873262377500497144172959572637630512207780360962
        );

        vk.IC[7] = Pairing.G1Point(
            3638528960970735074653385675510681349687682496258385248459462992524732263805,
            19827599334261471929021953664786047684020801747323901820332320895196040435196
        );

        vk.IC[8] = Pairing.G1Point(
            8163517276530828207003007367471373921638097877785141476697173049251842499010,
            11579466473086679359829471337364302848924982774226887421667461154392426408566
        );

        vk.IC[9] = Pairing.G1Point(
            17897391290813400398622058029694122678867677239961955250570247940262392862214,
            6136615397888756216685202143608243009642988577098567837056947606482885983060
        );

        vk.IC[10] = Pairing.G1Point(
            13363981918339270263875389313534670015095992334098826242905089546881823572108,
            656503940970761502542695606731037502505736713020737703602860116351126909721
        );

        vk.IC[11] = Pairing.G1Point(
            20725424337276746516628223713511205751620839306810638857159223337439147075584,
            11125813102863315547282055935296560367073108849036160938289557430560630316088
        );

        vk.IC[12] = Pairing.G1Point(
            14870930890904689140261649309649313116285548339468748437358059049429909667931,
            706472548942183778009647128112039294823461108594626252003508111656180302758
        );

        vk.IC[13] = Pairing.G1Point(
            496008602474722541882617406294379348888909312557782979911643670291903607859,
            21603000220803209667220780991303773023114924402627640141854264409883152370863
        );

        vk.IC[14] = Pairing.G1Point(
            361121337941540438859880020870856534215602982942650751841486563162511712764,
            10013819594767193610153718019277304447710980931189424157991217636296812213316
        );

        vk.IC[15] = Pairing.G1Point(
            2143009729783940358160713253455728481498514432122260770332152038045724833090,
            20087581440379048604008947204037328760432722650515630291445402310938177800352
        );

        vk.IC[16] = Pairing.G1Point(
            16731654525270169666286176102869295674264615648323122872187800543096693128916,
            7407119969245980834306662747434315101558866269180008444295987416379523886136
        );

        vk.IC[17] = Pairing.G1Point(
            15217123291710066120622787270662433946808192995138201778195726247938482732557,
            16052496246690325485601067535587978498471593345167451013775076236364464499504
        );

        vk.IC[18] = Pairing.G1Point(
            21719931496142668901729176811649838224579955242776306266370939248169491537078,
            4357683618578438747766002437775520411966609379043814401945789031679075101622
        );

        vk.IC[19] = Pairing.G1Point(
            5291695082895714108382088660011374636261974386780381249980086337020499895583,
            19812797022120710258752891336676782545361102125700418016684867592571569245690
        );

        vk.IC[20] = Pairing.G1Point(
            11829066565480477745324580767513514316983893950693258469811155386810451212840,
            20739533900141794545845203604537957746751755640441966814601896273419119547571
        );

        vk.IC[21] = Pairing.G1Point(
            9976853289690344646476036447204463318308661940373762210711765020067221869857,
            13277540167842934543752741696123549947033701951218946700220548871193389737077
        );

        vk.IC[22] = Pairing.G1Point(
            2815758203396220935198483497943296010961757277882205640261586772783326336889,
            2628330224015268759777852241390459329700921956412410815170098277887272797113
        );

        vk.IC[23] = Pairing.G1Point(
            3856396547269357983149176625404314285611056680986666905618518449693776875902,
            7271595086732831643842036227576490681153180862153811330930991202858912141311
        );

        vk.IC[24] = Pairing.G1Point(
            20813684323113455384150350197475078173608712983629035813769587072025729060378,
            5240416471251034223681830359766875035480000837055172959741301981592960512105
        );

        vk.IC[25] = Pairing.G1Point(
            2652617139185508100232502958928345356291132181565869957551188747612253813433,
            4060841158013574069115523140524357335652862509408938578595651611777381696545
        );

        vk.IC[26] = Pairing.G1Point(
            13916907905416583155231793004670905875499749037837884009246819849537195100512,
            10880017361812306142024652727138344014635521902005883396856455166334551181568
        );

        vk.IC[27] = Pairing.G1Point(
            19059646548179278763987256653739802817638094446117922695219747433615443784816,
            13380918668513520688333428800838898841090106766765693218322845350736075083597
        );

        vk.IC[28] = Pairing.G1Point(
            8396188053224095567882255367762412197107966469862229251565734292538502620707,
            16068625114754126467585747762909664589478885845728899722406361936132326255997
        );

        vk.IC[29] = Pairing.G1Point(
            6265696279339395833439569857032057399443946278839406127871555139223978264229,
            19503083926601140298267840691233153435404022682233673311694543687231890183835
        );

        vk.IC[30] = Pairing.G1Point(
            7111110408014320183343862227779415353814431834357699614785344473935016316897,
            19871244170386627009379546674003309134583518898392779637134866624684749943372
        );

    }
    function verify(uint[] memory input, Proof memory proof) internal view returns (uint) {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.IC.length,"verifier-bad-input");
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field,"verifier-gte-snark-scalar-field");
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.IC[i + 1], input[i]));
        }
        vk_x = Pairing.addition(vk_x, vk.IC[0]);
        if (!Pairing.pairingProd4(
            Pairing.negate(proof.A), proof.B,
            vk.alfa1, vk.beta2,
            vk_x, vk.gamma2,
            proof.C, vk.delta2
        )) return 1;
        return 0;
    }
    /// @return r  bool true if proof is valid
    function verifyProof(
            uint[2] memory a,
            uint[2][2] memory b,
            uint[2] memory c,
            uint[30] memory input
        ) public view returns (bool r) {
        Proof memory proof;
        proof.A = Pairing.G1Point(a[0], a[1]);
        proof.B = Pairing.G2Point([b[0][0], b[0][1]], [b[1][0], b[1][1]]);
        proof.C = Pairing.G1Point(c[0], c[1]);
        uint[] memory inputValues = new uint[](input.length);
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i];
        }
        if (verify(inputValues, proof) == 0) {
            return true;
        } else {
            return false;
        }
    }
}
