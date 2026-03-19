import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubNegotiationsComponent } from './hub-negotiations.component';

describe('HubNegotiationsComponent', () => {
  let component: HubNegotiationsComponent;
  let fixture: ComponentFixture<HubNegotiationsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubNegotiationsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubNegotiationsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
