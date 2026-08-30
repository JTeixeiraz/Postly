import "./index.css";
import { Composition } from "remotion";
import { Apresentacao, DURACAO_TOTAL } from "./Video";
import { FPS } from "./tokens";

export const RemotionRoot: React.FC = () => {
  return (
    <Composition
      id="Apresentacao"
      component={Apresentacao}
      durationInFrames={DURACAO_TOTAL}
      fps={FPS}
      width={1920}
      height={1080}
    />
  );
};
