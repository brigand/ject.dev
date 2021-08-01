import React, { useEffect } from 'react';
import pt from 'prop-types';
import styled from '@emotion/styled';
import { EventType } from '../EventType';
import { range } from '../utils/array';

const PER_GROUP = 10;

const LogLine = styled.pre`
  white-space: pre-wrap;
  max-width: 95%;
  padding-left: 0.4em;
  padding-right: 0.5em;
  line-height: 1.5;
  color: ${(props) =>
    props.method === 'error'
      ? 'var(--red)'
      : props.method === 'warn'
      ? 'var(--yellow)'
      : props.method === 'info'
      ? 'var(--blue)'
      : 'var(--ij-fg)'};

  &::before {
    content: '❯ ';
    display: inline-block;

    opacity: 0.5;
  }
`;

const ExecuteLine = styled.hr`
  border-color: rgba(255, 255, 255, 0.46);
`;

function Item({ item }) {
  return item.method === 'ject_execute' ? (
    <ExecuteLine />
  ) : (
    <LogLine method={item.method}>
      <code>{item.args.join(' ')}</code>
    </LogLine>
  );
}
Item.propTypes = {
  item: pt.shape({
    method: pt.string.isRequired,
    args: pt.arrayOf(pt.string).isRequired,
  }),
};
const ItemMemo = React.memo(Item);

function Group(props) {
  const [items, setItems] = React.useState([]);

  const take = () => {
    if (items.length >= PER_GROUP || !props.queue.length) {
      return;
    }

    setItems((current) => {
      if (current.length < PER_GROUP && props.queue.length) {
        const additional = props.queue.splice(0, PER_GROUP - current.length);
        if (additional.length) {
          return current.concat(additional);
        }
      }
      return current;
    });
  };

  props.pull.use(take);
  take();

  const ref = React.useRef();
  React.useEffect(() => {
    if (ref.current) {
      const scrollParent = ref.current.parentElement;
      scrollParent.scrollTo(0, scrollParent.scrollHeight - scrollParent.offsetHeight);
    }
  }, [items.length]);

  return (
    <div data-group={props.index} ref={ref}>
      {items.map((item, i) => (
        <ItemMemo key={i} item={item} />
      ))}
    </div>
  );
}

Group.propTypes = {
  index: pt.number.isRequired,
  pull: pt.instanceOf(EventType).isRequired,
  queue: pt.instanceOf(Array).isRequired,
};

const GroupMemo = React.memo(Group);

function Console(props) {
  const [pull] = React.useState(() => new EventType());
  const [queue] = React.useState(() => []);
  const [groups, setGroups] = React.useState(1);
  const rAF = React.useRef(null);
  const total = React.useRef(0);

  props.consoleMessage.use((event) => {
    if (
      event?.method === 'info' &&
      event.args.length === 1 &&
      event.args[0] === '[WDS] Live Reloading enabled.'
    ) {
      return;
    }

    total.current += 1;
    queue.push(event);
    if (!rAF.current) {
      rAF.current = requestAnimationFrame(() => {
        rAF.current = null;
        setGroups(Math.ceil((total.current + 1) / PER_GROUP));
        pull.emit();
      });
    }
  });

  useEffect(() => {
    if (props.submitCount > 1) {
      props.consoleMessage.emit({ method: 'ject_execute' });
    }
  }, [props.submitCount]);

  return (
    <div style={{ overflow: 'auto' }}>
      {range(0, groups).map((i) => (
        <GroupMemo key={i} index={i} queue={queue} pull={pull} />
      ))}
    </div>
  );
}

Console.propTypes = {
  consoleMessage: pt.instanceOf(EventType).isRequired,
  submitCount: pt.number.isRequired,
};

export default Console;
