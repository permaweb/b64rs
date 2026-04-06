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
