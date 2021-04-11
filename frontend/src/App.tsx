import React, { useState, useEffect } from "react";
import { Button, Table } from "semantic-ui-react";
import moment from "moment";

type Cassette = {
  uuid: string;
  name: string;
  url: string;
  created_at: string;
};

export default function App() {
  const [error, setError] = useState(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [items, setItems] = useState<Record<string, Cassette>>({});

  useEffect(() => {
    fetch("api/cassettes")
      .then((res) => res.json())
      .then(
        (result) => {
          setIsLoaded(true);
          setItems(result);
        },
        (error) => {
          setIsLoaded(true);
          setError(error);
        }
      );
  }, []);

  const play = (uuid: string) => {
    fetch("api/play/" + uuid);
  };
  const stop = () => {
    fetch("api/stop");
  };

  if (error) {
    return <div>Error: {error.message}</div>;
  } else if (!isLoaded) {
    return <div>Loading...</div>;
  } else {
    return (
      <div>
        <Button color="red" onClick={stop}>Stop</Button>
        <Table celled sortable>
          <Table.Header>
            <Table.Row>
              <Table.HeaderCell />
              <Table.HeaderCell>Name</Table.HeaderCell>
              <Table.HeaderCell>Created At</Table.HeaderCell>
              <Table.HeaderCell>Link</Table.HeaderCell>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {Object.values(items).sort((a, b) => a.created_at < b.created_at ? 1 : -1).map((item) => (
              <Table.Row key={item.uuid}>
                <Table.Cell><Button color="green" onClick={() => play(item.uuid)}>Play</Button></Table.Cell>
                <Table.Cell>{item.name}</Table.Cell>
                <Table.Cell>{moment(item.created_at).format("MMMM YYYY")}</Table.Cell>
                <Table.Cell><a href={item.url}>Link</a></Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table>
      </div>
    );
  }
}
