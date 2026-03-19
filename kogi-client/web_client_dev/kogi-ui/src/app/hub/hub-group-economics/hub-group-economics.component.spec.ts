import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubGroupEconomicsComponent } from './hub-group-economics.component';

describe('HubGroupEconomicsComponent', () => {
  let component: HubGroupEconomicsComponent;
  let fixture: ComponentFixture<HubGroupEconomicsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubGroupEconomicsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubGroupEconomicsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
