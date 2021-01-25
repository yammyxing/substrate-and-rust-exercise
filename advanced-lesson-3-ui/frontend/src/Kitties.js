import React, { useEffect, useState } from 'react';
import { Form, Grid } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

import KittyCards from './KittyCards';

export default function Kitties (props) {
  const { api, keyring } = useSubstrate();
  const { accountPair } = props;

  const [kittyCnt, setKittyCnt] = useState(0);
  const [kittyDNAs, setKittyDNAs] = useState([]);
  const [kittyOwners, setKittyOwners] = useState([]);
  const [kittyPrices, setKittyPrices] = useState([]);
  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('');

  const fetchKittyCnt = async () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    if (!api || !keyring) {
      return;
    }
    let unsubscribe = null
    // console.log("----:", api.query.kittiesModule)
    unsubscribe = await api.query.kittiesModule.kittiesCount(count => {
      setKittyCnt(count.toNumber());
    })

    return () => unsubscribe && unsubscribe();
  };

  const fetchKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubDna = null;
    let unsubOwner = null;
    let unsubPrice = null;

    const asyncFetch = async () => {
      const kittyIndices = [...Array(kittyCnt).keys()];

      unsubDna = await api.query.kittiesModule.kitties.multi(
        kittyIndices,
        dnas => setKittyDNAs(dnas.map(dna => dna.value.toU8a()))
      );

      unsubOwner = await api.query.kittiesModule.kittyOwners.multi(
        kittyIndices,
        owners => setKittyOwners(owners.map(owner => owner.toHuman()))
      );

      unsubPrice = await api.query.kittiesModule.kittyPrices.multi(
        kittyIndices,
        prices => setKittyPrices(prices.map(price => price.isSome && price.toHuman()))
      );
    };

    asyncFetch();

    // return the unsubscription cleanup function
    return () => {
      unsubDna && unsubDna();
      unsubOwner && unsubOwner();
      unsubPrice && unsubPrice();
    };
  };

  const populateKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    const kittyIndices = [...Array(kittyCnt).keys()];
    const kitties = kittyIndices.map(idx => ({
      id: idx,
      dna: kittyDNAs[ind],
      owner: kittyOwners[ind],
      price: kittyPrices[ind]
    }));
    setKitties(kitties);
  };

  useEffect(fetchKittyCnt, [api, keyring]);
  useEffect(fetchKitties, [api, kittyCnt]);
  useEffect(populateKitties, [kittyDNAs, kittyOwners]);

  return <Grid.Column width={16}>
    <h1>小毛孩</h1>
    <KittyCards kitties={kitties} accountPair={accountPair} setStatus={setStatus}/>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          accountPair={accountPair} label='创建小毛孩' type='SIGNED-TX' setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>;
}
