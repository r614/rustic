use rustic::modules::frame::RGB48Frame;

#[test]
    fn fixed_prediction_codec_test_1() {
		let codec = rustic::modules::codec::FixedPredictionCodec;

        let frame = RGB48Frame::open("src/testdata/tears_of_steel_12130.tif").unwrap();
        assert_eq!(frame.data.len(), 4096 * 1714 * 3); // 42,123,264 bytes uncompressed

        let mut encoded = Vec::new();
        frame.encode(&mut encoded, &codec).unwrap();
        assert_eq!(encoded.len(), 25526583);

        let decoded = frame.decode(&*encoded, frame.width, frame.height, &codec).unwrap();
        assert_eq!(frame == decoded, true);
    }

    #[test]
    fn fixed_prediction_codec_test_2() {
		let codec = rustic::modules::codec::FixedPredictionCodec;

        let frame = RGB48Frame::open("src/testdata/tears_of_steel_12209.tif").unwrap();
        assert_eq!(frame.data.len(), 4096 * 1714 * 3); // 42,123,264 bytes uncompressed

        let mut encoded = Vec::new();
        frame.encode(&mut encoded, &codec).unwrap();
        assert_eq!(encoded.len(), 28270586);

        let decoded = frame.decode(&*encoded, frame.width, frame.height, &codec).unwrap();
        assert_eq!(frame == decoded, true);
    }