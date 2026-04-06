-module(b64rs_test).
-include_lib("eunit/include/eunit.hrl").

roundtrip_test() ->
    Data = crypto:strong_rand_bytes(256 * 1024),
    ?assertEqual(Data, b64rs:decode(b64rs:encode(Data))).

empty_test() ->
    ?assertEqual(<<>>, b64rs:decode(b64rs:encode(<<>>))).

small_sizes_test() ->
    lists:foreach(fun(N) ->
        D = crypto:strong_rand_bytes(N),
        ?assertEqual(D, b64rs:decode(b64rs:encode(D)))
    end, [1, 2, 3, 4, 15, 16, 17, 255, 256, 1024]).

iolist_decode_test() ->
    Data = crypto:strong_rand_bytes(1024),
    Encoded = b64rs:encode(Data),
    ?assertEqual(Data, b64rs:decode([Encoded])).

iolist_encode_test() ->
    A = crypto:strong_rand_bytes(512),
    B = crypto:strong_rand_bytes(512),
    ?assertEqual(b64rs:encode(<<A/binary, B/binary>>), b64rs:encode([A, B])).

badarg_decode_test() ->
    ?assertError(badarg, b64rs:decode(123)).

badarg_encode_test() ->
    ?assertError(badarg, b64rs:encode(not_binary)).

malformed_input_test() ->
    ?assertError(badarg, b64rs:decode(<<"not!valid@base64$$$">>)).

standard_b64_decode_test() ->
    Std = base64:encode(crypto:strong_rand_bytes(256)),
    ?assertEqual(base64:decode(Std), b64rs:decode(Std)).

standard_padded_without_special_chars_test() ->
    ?assertEqual(<<"M">>, b64rs:decode(<<"TQ==">>)).

standard_unpadded_test() ->
    ?assertEqual(<<"M">>, b64rs:decode(<<"TQ">>)).

urlsafe_padded_test() ->
    ?assertEqual(<<251>>, b64rs:decode(<<"-w==">>)).

urlsafe_unpadded_test() ->
    ?assertEqual(<<251>>, b64rs:decode(<<"-w">>)).

mixed_alphabet_test() ->
    ?assertEqual(<<251, 255>>, b64rs:decode(<<"+_8=">>)).

whitespace_tolerant_decode_test() ->
    ?assertEqual(<<"Ma">>, b64rs:decode(<<"TW E=">>)).

invalid_length_remainder_test() ->
    ?assertError(badarg, b64rs:decode(<<"A">>)).

regression_nif_decode_sample_test() ->
    Encoded = <<"txy88T84q+0b8LKUqiVpU4xujEEArBvFaWvHXTlDty0b8VbAWOxp3Gg8">>,
    ?assertEqual(base64:decode(Encoded), b64rs:decode(Encoded)).
