#define OLC_PGE_APPLICATION

#include "includes/olcPixelGameEngine.h"

class OceanTest : public olc::PixelGameEngine {
  public:
    OceanTest() { sAppName = "Ocean Test"; }

  public:
    bool OnUserCreate() override { return true; }

    bool OnUserUpdate(float fElapsedTime) override {
        for (int x = 0; x < ScreenWidth(); x++)
            for (int y = 0; y < ScreenHeight(); y++)
                Draw(x, y,
                     olc::Pixel(rand() % 255, rand() % 255, rand() % 255));

        return true;
    }
};

int main() {
    OceanTest ot;

    if (ot.Construct(256, 240, 4, 4))
        ot.Start();
}
