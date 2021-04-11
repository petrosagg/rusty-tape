import React, { useState, useEffect } from "react";
import { Button, List } from "semantic-ui-react";

type Cassette = {
  uuid: string;
  name: string;
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
        <Button onClick={stop}>Stop</Button>
        <List>
          {Object.values(items).map((item) => (
            <List.Item key={item.uuid}>
              {item.name}
              <Button onClick={() => play(item.uuid)}>Play</Button>
            </List.Item>
          ))}
        </List>
      </div>
    );
  }
}
