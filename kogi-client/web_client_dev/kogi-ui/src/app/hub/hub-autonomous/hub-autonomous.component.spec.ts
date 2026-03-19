import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubAutonomousComponent } from './hub-autonomous.component';

describe('HubAutonomousComponent', () => {
  let component: HubAutonomousComponent;
  let fixture: ComponentFixture<HubAutonomousComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubAutonomousComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubAutonomousComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
